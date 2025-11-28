//! Category and CategoryGroup repository for JSON storage
//!
//! Manages loading and saving categories to budget.json

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::error::EnvelopeError;
use crate::models::{Category, CategoryGroup, CategoryGroupId, CategoryId};

use super::file_io::{read_json, write_json_atomic};

/// Serializable category data structure
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CategoryData {
    pub groups: Vec<CategoryGroup>,
    pub categories: Vec<Category>,
}

/// Repository for category and group persistence
pub struct CategoryRepository {
    path: PathBuf,
    groups: RwLock<HashMap<CategoryGroupId, CategoryGroup>>,
    categories: RwLock<HashMap<CategoryId, Category>>,
}

impl CategoryRepository {
    /// Create a new category repository
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            groups: RwLock::new(HashMap::new()),
            categories: RwLock::new(HashMap::new()),
        }
    }

    /// Load categories from disk
    pub fn load(&self) -> Result<(), EnvelopeError> {
        let file_data: CategoryData = read_json(&self.path)?;

        let mut groups = self
            .groups
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;
        let mut categories = self
            .categories
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        groups.clear();
        categories.clear();

        for group in file_data.groups {
            groups.insert(group.id, group);
        }

        for category in file_data.categories {
            categories.insert(category.id, category);
        }

        Ok(())
    }

    /// Save categories to disk
    pub fn save(&self) -> Result<(), EnvelopeError> {
        let groups = self
            .groups
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;
        let categories = self
            .categories
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let mut group_list: Vec<_> = groups.values().cloned().collect();
        group_list.sort_by_key(|g| g.sort_order);

        let mut category_list: Vec<_> = categories.values().cloned().collect();
        category_list.sort_by_key(|c| (c.sort_order, c.name.clone()));

        let file_data = CategoryData {
            groups: group_list,
            categories: category_list,
        };

        write_json_atomic(&self.path, &file_data)
    }

    // Group operations

    /// Get a group by ID
    pub fn get_group(&self, id: CategoryGroupId) -> Result<Option<CategoryGroup>, EnvelopeError> {
        let groups = self
            .groups
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        Ok(groups.get(&id).cloned())
    }

    /// Get all groups
    pub fn get_all_groups(&self) -> Result<Vec<CategoryGroup>, EnvelopeError> {
        let groups = self
            .groups
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let mut list: Vec<_> = groups.values().cloned().collect();
        list.sort_by_key(|g| g.sort_order);
        Ok(list)
    }

    /// Get a group by name
    pub fn get_group_by_name(&self, name: &str) -> Result<Option<CategoryGroup>, EnvelopeError> {
        let groups = self
            .groups
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let name_lower = name.to_lowercase();
        Ok(groups
            .values()
            .find(|g| g.name.to_lowercase() == name_lower)
            .cloned())
    }

    /// Insert or update a group
    pub fn upsert_group(&self, group: CategoryGroup) -> Result<(), EnvelopeError> {
        let mut groups = self
            .groups
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        groups.insert(group.id, group);
        Ok(())
    }

    /// Delete a group (and optionally its categories)
    pub fn delete_group(
        &self,
        id: CategoryGroupId,
        delete_categories: bool,
    ) -> Result<bool, EnvelopeError> {
        let mut groups = self
            .groups
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        if delete_categories {
            let mut categories = self.categories.write().map_err(|e| {
                EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
            })?;
            categories.retain(|_, c| c.group_id != id);
        }

        Ok(groups.remove(&id).is_some())
    }

    // Category operations

    /// Get a category by ID
    pub fn get_category(&self, id: CategoryId) -> Result<Option<Category>, EnvelopeError> {
        let categories = self
            .categories
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        Ok(categories.get(&id).cloned())
    }

    /// Get all categories
    pub fn get_all_categories(&self) -> Result<Vec<Category>, EnvelopeError> {
        let categories = self
            .categories
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let mut list: Vec<_> = categories.values().cloned().collect();
        list.sort_by_key(|c| (c.sort_order, c.name.clone()));
        Ok(list)
    }

    /// Get categories in a group
    pub fn get_categories_in_group(
        &self,
        group_id: CategoryGroupId,
    ) -> Result<Vec<Category>, EnvelopeError> {
        let categories = self
            .categories
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let mut list: Vec<_> = categories
            .values()
            .filter(|c| c.group_id == group_id)
            .cloned()
            .collect();
        list.sort_by_key(|c| (c.sort_order, c.name.clone()));
        Ok(list)
    }

    /// Get a category by name
    pub fn get_category_by_name(&self, name: &str) -> Result<Option<Category>, EnvelopeError> {
        let categories = self
            .categories
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let name_lower = name.to_lowercase();
        Ok(categories
            .values()
            .find(|c| c.name.to_lowercase() == name_lower)
            .cloned())
    }

    /// Insert or update a category
    pub fn upsert_category(&self, category: Category) -> Result<(), EnvelopeError> {
        let mut categories = self
            .categories
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        categories.insert(category.id, category);
        Ok(())
    }

    /// Delete a category
    pub fn delete_category(&self, id: CategoryId) -> Result<bool, EnvelopeError> {
        let mut categories = self
            .categories
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        Ok(categories.remove(&id).is_some())
    }

    /// Count groups
    pub fn group_count(&self) -> Result<usize, EnvelopeError> {
        let groups = self
            .groups
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;
        Ok(groups.len())
    }

    /// Count categories
    pub fn category_count(&self) -> Result<usize, EnvelopeError> {
        let categories = self
            .categories
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;
        Ok(categories.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, CategoryRepository) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("budget.json");
        let repo = CategoryRepository::new(path);
        (temp_dir, repo)
    }

    #[test]
    fn test_empty_load() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();
        assert_eq!(repo.group_count().unwrap(), 0);
        assert_eq!(repo.category_count().unwrap(), 0);
    }

    #[test]
    fn test_group_operations() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let group = CategoryGroup::new("Bills");
        let id = group.id;

        repo.upsert_group(group).unwrap();
        assert_eq!(repo.group_count().unwrap(), 1);

        let retrieved = repo.get_group(id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Bills");

        repo.delete_group(id, false).unwrap();
        assert_eq!(repo.group_count().unwrap(), 0);
    }

    #[test]
    fn test_category_operations() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let group = CategoryGroup::new("Bills");
        repo.upsert_group(group.clone()).unwrap();

        let category = Category::new("Rent", group.id);
        let cat_id = category.id;

        repo.upsert_category(category).unwrap();
        assert_eq!(repo.category_count().unwrap(), 1);

        let retrieved = repo.get_category(cat_id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Rent");

        let in_group = repo.get_categories_in_group(group.id).unwrap();
        assert_eq!(in_group.len(), 1);
    }

    #[test]
    fn test_save_and_reload() {
        let (temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let group = CategoryGroup::new("Bills");
        let category = Category::new("Rent", group.id);
        let cat_id = category.id;

        repo.upsert_group(group).unwrap();
        repo.upsert_category(category).unwrap();
        repo.save().unwrap();

        // Create new repo and load
        let path = temp_dir.path().join("budget.json");
        let repo2 = CategoryRepository::new(path);
        repo2.load().unwrap();

        assert_eq!(repo2.group_count().unwrap(), 1);
        assert_eq!(repo2.category_count().unwrap(), 1);

        let retrieved = repo2.get_category(cat_id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Rent");
    }

    #[test]
    fn test_get_by_name() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let group = CategoryGroup::new("My Bills");
        repo.upsert_group(group.clone()).unwrap();

        let category = Category::new("Monthly Rent", group.id);
        repo.upsert_category(category).unwrap();

        // Case insensitive
        let found_group = repo.get_group_by_name("my bills").unwrap();
        assert!(found_group.is_some());

        let found_cat = repo.get_category_by_name("MONTHLY RENT").unwrap();
        assert!(found_cat.is_some());
    }

    #[test]
    fn test_delete_group_with_categories() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let group = CategoryGroup::new("Bills");
        let group_id = group.id;
        repo.upsert_group(group.clone()).unwrap();

        let cat1 = Category::new("Rent", group.id);
        let cat2 = Category::new("Utilities", group.id);
        repo.upsert_category(cat1).unwrap();
        repo.upsert_category(cat2).unwrap();

        assert_eq!(repo.category_count().unwrap(), 2);

        repo.delete_group(group_id, true).unwrap();

        assert_eq!(repo.group_count().unwrap(), 0);
        assert_eq!(repo.category_count().unwrap(), 0);
    }
}
