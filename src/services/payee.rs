//! Payee service
//!
//! Provides business logic for payee management including auto-suggestion,
//! category learning, and fuzzy matching.

use crate::audit::EntityType;
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{CategoryId, Payee, PayeeId};
use crate::storage::Storage;

/// Service for payee management
pub struct PayeeService<'a> {
    storage: &'a Storage,
}

impl<'a> PayeeService<'a> {
    /// Create a new payee service
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Create a new payee
    pub fn create(&self, name: &str) -> EnvelopeResult<Payee> {
        let name = name.trim();
        if name.is_empty() {
            return Err(EnvelopeError::Validation("Payee name cannot be empty".into()));
        }

        // Check for duplicate
        if self.storage.payees.get_by_name(name)?.is_some() {
            return Err(EnvelopeError::Duplicate {
                entity_type: "Payee",
                identifier: name.to_string(),
            });
        }

        let mut payee = Payee::new(name);
        payee.manual = true;

        // Validate
        payee
            .validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        // Save
        self.storage.payees.upsert(payee.clone())?;
        self.storage.payees.save()?;

        // Audit log
        self.storage.log_create(
            EntityType::Payee,
            payee.id.to_string(),
            Some(payee.name.clone()),
            &payee,
        )?;

        Ok(payee)
    }

    /// Create a payee with a default category
    pub fn create_with_category(
        &self,
        name: &str,
        category_id: CategoryId,
    ) -> EnvelopeResult<Payee> {
        let name = name.trim();
        if name.is_empty() {
            return Err(EnvelopeError::Validation("Payee name cannot be empty".into()));
        }

        // Verify category exists
        self.storage
            .categories
            .get_category(category_id)?
            .ok_or_else(|| EnvelopeError::category_not_found(category_id.to_string()))?;

        // Check for duplicate
        if self.storage.payees.get_by_name(name)?.is_some() {
            return Err(EnvelopeError::Duplicate {
                entity_type: "Payee",
                identifier: name.to_string(),
            });
        }

        let payee = Payee::with_default_category(name, category_id);

        // Validate
        payee
            .validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        // Save
        self.storage.payees.upsert(payee.clone())?;
        self.storage.payees.save()?;

        // Audit log
        self.storage.log_create(
            EntityType::Payee,
            payee.id.to_string(),
            Some(payee.name.clone()),
            &payee,
        )?;

        Ok(payee)
    }

    /// Get a payee by ID
    pub fn get(&self, id: PayeeId) -> EnvelopeResult<Option<Payee>> {
        self.storage.payees.get(id)
    }

    /// Get a payee by name (case-insensitive)
    pub fn get_by_name(&self, name: &str) -> EnvelopeResult<Option<Payee>> {
        self.storage.payees.get_by_name(name)
    }

    /// Find a payee by ID or name
    pub fn find(&self, identifier: &str) -> EnvelopeResult<Option<Payee>> {
        // Try by name first
        if let Some(payee) = self.storage.payees.get_by_name(identifier)? {
            return Ok(Some(payee));
        }

        // Try parsing as ID
        if let Ok(id) = identifier.parse::<PayeeId>() {
            return self.storage.payees.get(id);
        }

        Ok(None)
    }

    /// Get or create a payee by name
    pub fn get_or_create(&self, name: &str) -> EnvelopeResult<Payee> {
        self.storage.payees.get_or_create(name)
    }

    /// List all payees
    pub fn list(&self) -> EnvelopeResult<Vec<Payee>> {
        self.storage.payees.get_all()
    }

    /// Search payees by name (fuzzy match)
    pub fn search(&self, query: &str, limit: usize) -> EnvelopeResult<Vec<Payee>> {
        self.storage.payees.search(query, limit)
    }

    /// Suggest payees matching a partial name
    pub fn suggest(&self, partial: &str) -> EnvelopeResult<Vec<Payee>> {
        self.storage.payees.search(partial, 10)
    }

    /// Get the suggested category for a payee
    pub fn get_suggested_category(&self, payee_name: &str) -> EnvelopeResult<Option<CategoryId>> {
        if let Some(payee) = self.storage.payees.get_by_name(payee_name)? {
            Ok(payee.suggested_category())
        } else {
            Ok(None)
        }
    }

    /// Set the default category for a payee
    pub fn set_default_category(
        &self,
        id: PayeeId,
        category_id: CategoryId,
    ) -> EnvelopeResult<Payee> {
        let mut payee = self
            .storage
            .payees
            .get(id)?
            .ok_or_else(|| EnvelopeError::payee_not_found(id.to_string()))?;

        // Verify category exists
        self.storage
            .categories
            .get_category(category_id)?
            .ok_or_else(|| EnvelopeError::category_not_found(category_id.to_string()))?;

        let before = payee.clone();
        payee.set_default_category(category_id);

        // Save
        self.storage.payees.upsert(payee.clone())?;
        self.storage.payees.save()?;

        // Audit log
        self.storage.log_update(
            EntityType::Payee,
            payee.id.to_string(),
            Some(payee.name.clone()),
            &before,
            &payee,
            Some(format!(
                "default_category: {:?} -> {:?}",
                before.default_category_id, payee.default_category_id
            )),
        )?;

        Ok(payee)
    }

    /// Clear the default category for a payee
    pub fn clear_default_category(&self, id: PayeeId) -> EnvelopeResult<Payee> {
        let mut payee = self
            .storage
            .payees
            .get(id)?
            .ok_or_else(|| EnvelopeError::payee_not_found(id.to_string()))?;

        let before = payee.clone();
        payee.clear_default_category();

        // Save
        self.storage.payees.upsert(payee.clone())?;
        self.storage.payees.save()?;

        // Audit log
        self.storage.log_update(
            EntityType::Payee,
            payee.id.to_string(),
            Some(payee.name.clone()),
            &before,
            &payee,
            Some(format!(
                "default_category: {:?} -> None",
                before.default_category_id
            )),
        )?;

        Ok(payee)
    }

    /// Record a category usage for a payee (for learning)
    pub fn record_category_usage(
        &self,
        payee_id: PayeeId,
        category_id: CategoryId,
    ) -> EnvelopeResult<()> {
        if let Some(mut payee) = self.storage.payees.get(payee_id)? {
            payee.record_category_usage(category_id);
            self.storage.payees.upsert(payee)?;
            self.storage.payees.save()?;
        }
        Ok(())
    }

    /// Delete a payee
    pub fn delete(&self, id: PayeeId) -> EnvelopeResult<Payee> {
        let payee = self
            .storage
            .payees
            .get(id)?
            .ok_or_else(|| EnvelopeError::payee_not_found(id.to_string()))?;

        self.storage.payees.delete(id)?;
        self.storage.payees.save()?;

        // Audit log
        self.storage.log_delete(
            EntityType::Payee,
            id.to_string(),
            Some(payee.name.clone()),
            &payee,
        )?;

        Ok(payee)
    }

    /// Rename a payee
    pub fn rename(&self, id: PayeeId, new_name: &str) -> EnvelopeResult<Payee> {
        let new_name = new_name.trim();
        if new_name.is_empty() {
            return Err(EnvelopeError::Validation("Payee name cannot be empty".into()));
        }

        let mut payee = self
            .storage
            .payees
            .get(id)?
            .ok_or_else(|| EnvelopeError::payee_not_found(id.to_string()))?;

        // Check for duplicate (excluding self)
        if let Some(existing) = self.storage.payees.get_by_name(new_name)? {
            if existing.id != id {
                return Err(EnvelopeError::Duplicate {
                    entity_type: "Payee",
                    identifier: new_name.to_string(),
                });
            }
        }

        let before = payee.clone();
        payee.name = new_name.to_string();
        payee.updated_at = chrono::Utc::now();

        // Validate
        payee
            .validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        // Save
        self.storage.payees.upsert(payee.clone())?;
        self.storage.payees.save()?;

        // Audit log
        self.storage.log_update(
            EntityType::Payee,
            payee.id.to_string(),
            Some(payee.name.clone()),
            &before,
            &payee,
            Some(format!("name: '{}' -> '{}'", before.name, payee.name)),
        )?;

        Ok(payee)
    }

    /// Count payees
    pub fn count(&self) -> EnvelopeResult<usize> {
        self.storage.payees.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::EnvelopePaths;
    use crate::models::{Category, CategoryGroup};
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, Storage) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        let mut storage = Storage::new(paths).unwrap();
        storage.load_all().unwrap();
        (temp_dir, storage)
    }

    fn setup_test_category(storage: &Storage) -> CategoryId {
        let group = CategoryGroup::new("Test Group");
        storage.categories.upsert_group(group.clone()).unwrap();

        let category = Category::new("Groceries", group.id);
        let category_id = category.id;
        storage.categories.upsert_category(category).unwrap();
        storage.categories.save().unwrap();

        category_id
    }

    #[test]
    fn test_create_payee() {
        let (_temp_dir, storage) = create_test_storage();
        let service = PayeeService::new(&storage);

        let payee = service.create("Test Store").unwrap();
        assert_eq!(payee.name, "Test Store");
        assert!(payee.manual);
    }

    #[test]
    fn test_create_with_category() {
        let (_temp_dir, storage) = create_test_storage();
        let category_id = setup_test_category(&storage);
        let service = PayeeService::new(&storage);

        let payee = service
            .create_with_category("Grocery Store", category_id)
            .unwrap();

        assert_eq!(payee.default_category_id, Some(category_id));
        assert!(payee.manual);
    }

    #[test]
    fn test_duplicate_payee() {
        let (_temp_dir, storage) = create_test_storage();
        let service = PayeeService::new(&storage);

        service.create("Test Store").unwrap();
        let result = service.create("test store"); // case insensitive
        assert!(matches!(result, Err(EnvelopeError::Duplicate { .. })));
    }

    #[test]
    fn test_search_payees() {
        let (_temp_dir, storage) = create_test_storage();
        let service = PayeeService::new(&storage);

        service.create("Grocery Store").unwrap();
        service.create("Gas Station").unwrap();
        service.create("Restaurant").unwrap();

        let results = service.search("groc", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "Grocery Store");
    }

    #[test]
    fn test_category_learning() {
        let (_temp_dir, storage) = create_test_storage();
        let category_id = setup_test_category(&storage);
        let service = PayeeService::new(&storage);

        let payee = service.create("Learning Store").unwrap();

        // Record some usage
        service
            .record_category_usage(payee.id, category_id)
            .unwrap();
        service
            .record_category_usage(payee.id, category_id)
            .unwrap();

        // Check suggested category
        let suggested = service
            .get_suggested_category("Learning Store")
            .unwrap();
        assert_eq!(suggested, Some(category_id));
    }

    #[test]
    fn test_set_default_category() {
        let (_temp_dir, storage) = create_test_storage();
        let category_id = setup_test_category(&storage);
        let service = PayeeService::new(&storage);

        let payee = service.create("Test Payee").unwrap();
        assert!(payee.default_category_id.is_none());

        let updated = service
            .set_default_category(payee.id, category_id)
            .unwrap();
        assert_eq!(updated.default_category_id, Some(category_id));
        assert!(updated.manual);
    }

    #[test]
    fn test_delete_payee() {
        let (_temp_dir, storage) = create_test_storage();
        let service = PayeeService::new(&storage);

        let payee = service.create("To Delete").unwrap();
        assert_eq!(service.count().unwrap(), 1);

        service.delete(payee.id).unwrap();
        assert_eq!(service.count().unwrap(), 0);
    }

    #[test]
    fn test_rename_payee() {
        let (_temp_dir, storage) = create_test_storage();
        let service = PayeeService::new(&storage);

        let payee = service.create("Old Name").unwrap();
        let renamed = service.rename(payee.id, "New Name").unwrap();

        assert_eq!(renamed.name, "New Name");
        assert!(service.get_by_name("Old Name").unwrap().is_none());
        assert!(service.get_by_name("New Name").unwrap().is_some());
    }
}
