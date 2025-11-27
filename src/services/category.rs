//! Category service
//!
//! Provides business logic for category and category group management
//! including CRUD operations, reordering, and moving categories between groups.

use crate::audit::EntityType;
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{Category, CategoryGroup, CategoryGroupId, CategoryId};
use crate::storage::Storage;

/// Service for category management
pub struct CategoryService<'a> {
    storage: &'a Storage,
}

/// A category group with its categories
#[derive(Debug, Clone)]
pub struct CategoryGroupWithCategories {
    pub group: CategoryGroup,
    pub categories: Vec<Category>,
}

impl<'a> CategoryService<'a> {
    /// Create a new category service
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    // === Group Operations ===

    /// Create a new category group
    pub fn create_group(&self, name: &str) -> EnvelopeResult<CategoryGroup> {
        let name = name.trim();
        if name.is_empty() {
            return Err(EnvelopeError::Validation(
                "Category group name cannot be empty".into(),
            ));
        }

        // Check for duplicate name
        if self.storage.categories.get_group_by_name(name)?.is_some() {
            return Err(EnvelopeError::Duplicate {
                entity_type: "Category Group",
                identifier: name.to_string(),
            });
        }

        // Get max sort order
        let groups = self.storage.categories.get_all_groups()?;
        let max_order = groups.iter().map(|g| g.sort_order).max().unwrap_or(-1);

        let mut group = CategoryGroup::new(name);
        group.sort_order = max_order + 1;

        group
            .validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        // Save
        self.storage.categories.upsert_group(group.clone())?;
        self.storage.categories.save()?;

        // Audit log
        self.storage.log_create(
            EntityType::CategoryGroup,
            group.id.to_string(),
            Some(group.name.clone()),
            &group,
        )?;

        Ok(group)
    }

    /// Get a group by ID
    pub fn get_group(&self, id: CategoryGroupId) -> EnvelopeResult<Option<CategoryGroup>> {
        self.storage.categories.get_group(id)
    }

    /// Get a group by name (case-insensitive)
    pub fn get_group_by_name(&self, name: &str) -> EnvelopeResult<Option<CategoryGroup>> {
        self.storage.categories.get_group_by_name(name)
    }

    /// Find a group by name or ID string
    pub fn find_group(&self, identifier: &str) -> EnvelopeResult<Option<CategoryGroup>> {
        // Try by name first
        if let Some(group) = self.storage.categories.get_group_by_name(identifier)? {
            return Ok(Some(group));
        }

        // Try parsing as ID
        if let Ok(id) = identifier.parse::<CategoryGroupId>() {
            return self.storage.categories.get_group(id);
        }

        Ok(None)
    }

    /// List all groups
    pub fn list_groups(&self) -> EnvelopeResult<Vec<CategoryGroup>> {
        self.storage.categories.get_all_groups()
    }

    /// List all groups with their categories
    pub fn list_groups_with_categories(&self) -> EnvelopeResult<Vec<CategoryGroupWithCategories>> {
        let groups = self.storage.categories.get_all_groups()?;
        let mut result = Vec::with_capacity(groups.len());

        for group in groups {
            let categories = self.storage.categories.get_categories_in_group(group.id)?;
            result.push(CategoryGroupWithCategories { group, categories });
        }

        Ok(result)
    }

    /// Update a group's name
    pub fn update_group(&self, id: CategoryGroupId, name: Option<&str>) -> EnvelopeResult<CategoryGroup> {
        let mut group = self
            .storage
            .categories
            .get_group(id)?
            .ok_or_else(|| EnvelopeError::NotFound {
                entity_type: "Category Group",
                identifier: id.to_string(),
            })?;

        let before = group.clone();

        if let Some(new_name) = name {
            let new_name = new_name.trim();
            if new_name.is_empty() {
                return Err(EnvelopeError::Validation(
                    "Category group name cannot be empty".into(),
                ));
            }

            // Check for duplicate
            if let Some(existing) = self.storage.categories.get_group_by_name(new_name)? {
                if existing.id != id {
                    return Err(EnvelopeError::Duplicate {
                        entity_type: "Category Group",
                        identifier: new_name.to_string(),
                    });
                }
            }

            group.name = new_name.to_string();
        }

        group.updated_at = chrono::Utc::now();
        group
            .validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        self.storage.categories.upsert_group(group.clone())?;
        self.storage.categories.save()?;

        // Audit
        if before.name != group.name {
            self.storage.log_update(
                EntityType::CategoryGroup,
                group.id.to_string(),
                Some(group.name.clone()),
                &before,
                &group,
                Some(format!("name: {} -> {}", before.name, group.name)),
            )?;
        }

        Ok(group)
    }

    /// Delete a group
    ///
    /// If the group has categories, they must be moved or deleted first
    /// unless force_delete_categories is true.
    pub fn delete_group(&self, id: CategoryGroupId, force_delete_categories: bool) -> EnvelopeResult<()> {
        let group = self
            .storage
            .categories
            .get_group(id)?
            .ok_or_else(|| EnvelopeError::NotFound {
                entity_type: "Category Group",
                identifier: id.to_string(),
            })?;

        let categories = self.storage.categories.get_categories_in_group(id)?;
        if !categories.is_empty() && !force_delete_categories {
            return Err(EnvelopeError::Validation(format!(
                "Cannot delete group '{}' - it contains {} categories. Use --force to delete them.",
                group.name,
                categories.len()
            )));
        }

        self.storage.categories.delete_group(id, force_delete_categories)?;
        self.storage.categories.save()?;

        // Audit
        self.storage.log_delete(
            EntityType::CategoryGroup,
            group.id.to_string(),
            Some(group.name.clone()),
            &group,
        )?;

        Ok(())
    }

    /// Reorder groups
    pub fn reorder_groups(&self, order: &[CategoryGroupId]) -> EnvelopeResult<()> {
        for (i, &id) in order.iter().enumerate() {
            if let Some(mut group) = self.storage.categories.get_group(id)? {
                group.sort_order = i as i32;
                group.updated_at = chrono::Utc::now();
                self.storage.categories.upsert_group(group)?;
            }
        }
        self.storage.categories.save()?;
        Ok(())
    }

    // === Category Operations ===

    /// Create a new category in a group
    pub fn create_category(&self, name: &str, group_id: CategoryGroupId) -> EnvelopeResult<Category> {
        let name = name.trim();
        if name.is_empty() {
            return Err(EnvelopeError::Validation("Category name cannot be empty".into()));
        }

        // Verify group exists
        if self.storage.categories.get_group(group_id)?.is_none() {
            return Err(EnvelopeError::NotFound {
                entity_type: "Category Group",
                identifier: group_id.to_string(),
            });
        }

        // Check for duplicate name (globally)
        if self.storage.categories.get_category_by_name(name)?.is_some() {
            return Err(EnvelopeError::Duplicate {
                entity_type: "Category",
                identifier: name.to_string(),
            });
        }

        // Get max sort order in group
        let categories = self.storage.categories.get_categories_in_group(group_id)?;
        let max_order = categories.iter().map(|c| c.sort_order).max().unwrap_or(-1);

        let mut category = Category::new(name, group_id);
        category.sort_order = max_order + 1;

        category
            .validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        self.storage.categories.upsert_category(category.clone())?;
        self.storage.categories.save()?;

        // Audit
        self.storage.log_create(
            EntityType::Category,
            category.id.to_string(),
            Some(category.name.clone()),
            &category,
        )?;

        Ok(category)
    }

    /// Get a category by ID
    pub fn get_category(&self, id: CategoryId) -> EnvelopeResult<Option<Category>> {
        self.storage.categories.get_category(id)
    }

    /// Get a category by name (case-insensitive)
    pub fn get_category_by_name(&self, name: &str) -> EnvelopeResult<Option<Category>> {
        self.storage.categories.get_category_by_name(name)
    }

    /// Find a category by name or ID string
    pub fn find_category(&self, identifier: &str) -> EnvelopeResult<Option<Category>> {
        // Try by name first
        if let Some(category) = self.storage.categories.get_category_by_name(identifier)? {
            return Ok(Some(category));
        }

        // Try parsing as ID
        if let Ok(id) = identifier.parse::<CategoryId>() {
            return self.storage.categories.get_category(id);
        }

        Ok(None)
    }

    /// List all categories
    pub fn list_categories(&self) -> EnvelopeResult<Vec<Category>> {
        self.storage.categories.get_all_categories()
    }

    /// List categories in a group
    pub fn list_categories_in_group(&self, group_id: CategoryGroupId) -> EnvelopeResult<Vec<Category>> {
        self.storage.categories.get_categories_in_group(group_id)
    }

    /// Update a category
    pub fn update_category(
        &self,
        id: CategoryId,
        name: Option<&str>,
        goal: Option<i64>,
        clear_goal: bool,
    ) -> EnvelopeResult<Category> {
        let mut category = self
            .storage
            .categories
            .get_category(id)?
            .ok_or_else(|| EnvelopeError::category_not_found(id.to_string()))?;

        let before = category.clone();

        if let Some(new_name) = name {
            let new_name = new_name.trim();
            if new_name.is_empty() {
                return Err(EnvelopeError::Validation("Category name cannot be empty".into()));
            }

            // Check for duplicate
            if let Some(existing) = self.storage.categories.get_category_by_name(new_name)? {
                if existing.id != id {
                    return Err(EnvelopeError::Duplicate {
                        entity_type: "Category",
                        identifier: new_name.to_string(),
                    });
                }
            }

            category.name = new_name.to_string();
        }

        if clear_goal {
            category.clear_goal();
        } else if let Some(goal_amount) = goal {
            category.set_goal(goal_amount);
        }

        category.updated_at = chrono::Utc::now();
        category
            .validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        self.storage.categories.upsert_category(category.clone())?;
        self.storage.categories.save()?;

        // Audit
        let mut changes = Vec::new();
        if before.name != category.name {
            changes.push(format!("name: {} -> {}", before.name, category.name));
        }
        if before.goal_amount != category.goal_amount {
            changes.push(format!(
                "goal: {:?} -> {:?}",
                before.goal_amount, category.goal_amount
            ));
        }

        if !changes.is_empty() {
            self.storage.log_update(
                EntityType::Category,
                category.id.to_string(),
                Some(category.name.clone()),
                &before,
                &category,
                Some(changes.join(", ")),
            )?;
        }

        Ok(category)
    }

    /// Move a category to a different group
    pub fn move_category(&self, id: CategoryId, new_group_id: CategoryGroupId) -> EnvelopeResult<Category> {
        let mut category = self
            .storage
            .categories
            .get_category(id)?
            .ok_or_else(|| EnvelopeError::category_not_found(id.to_string()))?;

        // Verify new group exists
        let new_group = self
            .storage
            .categories
            .get_group(new_group_id)?
            .ok_or_else(|| EnvelopeError::NotFound {
                entity_type: "Category Group",
                identifier: new_group_id.to_string(),
            })?;

        let before = category.clone();
        let old_group = self.storage.categories.get_group(category.group_id)?;

        category.move_to_group(new_group_id);

        // Update sort order to be last in new group
        let categories = self.storage.categories.get_categories_in_group(new_group_id)?;
        let max_order = categories.iter().map(|c| c.sort_order).max().unwrap_or(-1);
        category.sort_order = max_order + 1;

        self.storage.categories.upsert_category(category.clone())?;
        self.storage.categories.save()?;

        // Audit
        self.storage.log_update(
            EntityType::Category,
            category.id.to_string(),
            Some(category.name.clone()),
            &before,
            &category,
            Some(format!(
                "moved from '{}' to '{}'",
                old_group.map(|g| g.name).unwrap_or_else(|| "Unknown".into()),
                new_group.name
            )),
        )?;

        Ok(category)
    }

    /// Delete a category
    pub fn delete_category(&self, id: CategoryId) -> EnvelopeResult<()> {
        let category = self
            .storage
            .categories
            .get_category(id)?
            .ok_or_else(|| EnvelopeError::category_not_found(id.to_string()))?;

        // TODO: Check for budget allocations and transactions using this category
        // For now, just delete

        self.storage.categories.delete_category(id)?;
        self.storage.categories.save()?;

        // Audit
        self.storage.log_delete(
            EntityType::Category,
            category.id.to_string(),
            Some(category.name.clone()),
            &category,
        )?;

        Ok(())
    }

    /// Reorder categories within a group
    pub fn reorder_categories(&self, group_id: CategoryGroupId, order: &[CategoryId]) -> EnvelopeResult<()> {
        for (i, &id) in order.iter().enumerate() {
            if let Some(mut category) = self.storage.categories.get_category(id)? {
                if category.group_id == group_id {
                    category.sort_order = i as i32;
                    category.updated_at = chrono::Utc::now();
                    self.storage.categories.upsert_category(category)?;
                }
            }
        }
        self.storage.categories.save()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::EnvelopePaths;
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, Storage) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        let mut storage = Storage::new(paths).unwrap();
        storage.load_all().unwrap();
        (temp_dir, storage)
    }

    #[test]
    fn test_create_group() {
        let (_temp_dir, storage) = create_test_storage();
        let service = CategoryService::new(&storage);

        let group = service.create_group("Bills").unwrap();
        assert_eq!(group.name, "Bills");
        assert_eq!(group.sort_order, 0);
    }

    #[test]
    fn test_create_duplicate_group() {
        let (_temp_dir, storage) = create_test_storage();
        let service = CategoryService::new(&storage);

        service.create_group("Bills").unwrap();
        let result = service.create_group("Bills");
        assert!(matches!(result, Err(EnvelopeError::Duplicate { .. })));
    }

    #[test]
    fn test_create_category() {
        let (_temp_dir, storage) = create_test_storage();
        let service = CategoryService::new(&storage);

        let group = service.create_group("Bills").unwrap();
        let category = service.create_category("Rent", group.id).unwrap();

        assert_eq!(category.name, "Rent");
        assert_eq!(category.group_id, group.id);
    }

    #[test]
    fn test_list_groups_with_categories() {
        let (_temp_dir, storage) = create_test_storage();
        let service = CategoryService::new(&storage);

        let group = service.create_group("Bills").unwrap();
        service.create_category("Rent", group.id).unwrap();
        service.create_category("Electric", group.id).unwrap();

        let result = service.list_groups_with_categories().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].categories.len(), 2);
    }

    #[test]
    fn test_move_category() {
        let (_temp_dir, storage) = create_test_storage();
        let service = CategoryService::new(&storage);

        let bills = service.create_group("Bills").unwrap();
        let needs = service.create_group("Needs").unwrap();

        let category = service.create_category("Groceries", bills.id).unwrap();
        assert_eq!(category.group_id, bills.id);

        let moved = service.move_category(category.id, needs.id).unwrap();
        assert_eq!(moved.group_id, needs.id);
    }

    #[test]
    fn test_delete_category() {
        let (_temp_dir, storage) = create_test_storage();
        let service = CategoryService::new(&storage);

        let group = service.create_group("Bills").unwrap();
        let category = service.create_category("Rent", group.id).unwrap();

        assert!(service.get_category(category.id).unwrap().is_some());

        service.delete_category(category.id).unwrap();

        assert!(service.get_category(category.id).unwrap().is_none());
    }

    #[test]
    fn test_find_category() {
        let (_temp_dir, storage) = create_test_storage();
        let service = CategoryService::new(&storage);

        let group = service.create_group("Bills").unwrap();
        let category = service.create_category("Monthly Rent", group.id).unwrap();

        // Find by name (case insensitive)
        let found = service.find_category("monthly rent").unwrap().unwrap();
        assert_eq!(found.id, category.id);
    }
}
