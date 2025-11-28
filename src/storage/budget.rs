//! Budget allocation repository for JSON storage
//!
//! Manages loading and saving budget allocations (shares budget.json with categories)

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::error::EnvelopeError;
use crate::models::{BudgetAllocation, BudgetPeriod, CategoryId};

use super::file_io::{read_json, write_json_atomic};

/// Serializable budget data (extends CategoryData)
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct BudgetData {
    #[serde(default)]
    allocations: Vec<BudgetAllocation>,
}

/// Composite key for budget allocations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AllocationKey {
    pub category_id: CategoryId,
    pub period: BudgetPeriod,
}

impl AllocationKey {
    pub fn new(category_id: CategoryId, period: BudgetPeriod) -> Self {
        Self {
            category_id,
            period,
        }
    }
}

/// Repository for budget allocation persistence
pub struct BudgetRepository {
    path: PathBuf,
    allocations: RwLock<HashMap<AllocationKey, BudgetAllocation>>,
}

impl BudgetRepository {
    /// Create a new budget repository
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            allocations: RwLock::new(HashMap::new()),
        }
    }

    /// Load allocations from disk
    pub fn load(&self) -> Result<(), EnvelopeError> {
        let file_data: BudgetData = read_json(&self.path)?;

        let mut allocations = self
            .allocations
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        allocations.clear();
        for alloc in file_data.allocations {
            let key = AllocationKey::new(alloc.category_id, alloc.period.clone());
            allocations.insert(key, alloc);
        }

        Ok(())
    }

    /// Save allocations to disk
    pub fn save(&self) -> Result<(), EnvelopeError> {
        let allocations = self
            .allocations
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let mut alloc_list: Vec<_> = allocations.values().cloned().collect();
        alloc_list.sort_by(|a, b| a.period.cmp(&b.period));

        let file_data = BudgetData {
            allocations: alloc_list,
        };

        write_json_atomic(&self.path, &file_data)
    }

    /// Get an allocation for a category and period
    pub fn get(
        &self,
        category_id: CategoryId,
        period: &BudgetPeriod,
    ) -> Result<Option<BudgetAllocation>, EnvelopeError> {
        let allocations = self
            .allocations
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let key = AllocationKey::new(category_id, period.clone());
        Ok(allocations.get(&key).cloned())
    }

    /// Get or create an allocation (returns default if not found)
    pub fn get_or_default(
        &self,
        category_id: CategoryId,
        period: &BudgetPeriod,
    ) -> Result<BudgetAllocation, EnvelopeError> {
        if let Some(alloc) = self.get(category_id, period)? {
            Ok(alloc)
        } else {
            Ok(BudgetAllocation::new(category_id, period.clone()))
        }
    }

    /// Get all allocations for a period
    pub fn get_for_period(
        &self,
        period: &BudgetPeriod,
    ) -> Result<Vec<BudgetAllocation>, EnvelopeError> {
        let allocations = self
            .allocations
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        Ok(allocations
            .values()
            .filter(|a| &a.period == period)
            .cloned()
            .collect())
    }

    /// Get all allocations for a category
    pub fn get_for_category(
        &self,
        category_id: CategoryId,
    ) -> Result<Vec<BudgetAllocation>, EnvelopeError> {
        let allocations = self
            .allocations
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let mut list: Vec<_> = allocations
            .values()
            .filter(|a| a.category_id == category_id)
            .cloned()
            .collect();
        list.sort_by(|a, b| a.period.cmp(&b.period));
        Ok(list)
    }

    /// Insert or update an allocation
    pub fn upsert(&self, allocation: BudgetAllocation) -> Result<(), EnvelopeError> {
        let mut allocations = self
            .allocations
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        let key = AllocationKey::new(allocation.category_id, allocation.period.clone());
        allocations.insert(key, allocation);
        Ok(())
    }

    /// Delete an allocation
    pub fn delete(
        &self,
        category_id: CategoryId,
        period: &BudgetPeriod,
    ) -> Result<bool, EnvelopeError> {
        let mut allocations = self
            .allocations
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        let key = AllocationKey::new(category_id, period.clone());
        Ok(allocations.remove(&key).is_some())
    }

    /// Delete all allocations for a category
    pub fn delete_for_category(&self, category_id: CategoryId) -> Result<usize, EnvelopeError> {
        let mut allocations = self
            .allocations
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        let initial_count = allocations.len();
        allocations.retain(|k, _| k.category_id != category_id);
        Ok(initial_count - allocations.len())
    }

    /// Count allocations
    pub fn count(&self) -> Result<usize, EnvelopeError> {
        let allocations = self
            .allocations
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;
        Ok(allocations.len())
    }

    /// Get all allocations
    pub fn get_all(&self) -> Result<Vec<BudgetAllocation>, EnvelopeError> {
        let allocations = self
            .allocations
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let mut list: Vec<_> = allocations.values().cloned().collect();
        list.sort_by(|a, b| a.period.cmp(&b.period));
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Money;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, BudgetRepository) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("budget.json");
        let repo = BudgetRepository::new(path);
        (temp_dir, repo)
    }

    fn test_period() -> BudgetPeriod {
        BudgetPeriod::monthly(2025, 1)
    }

    #[test]
    fn test_empty_load() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();
        assert_eq!(repo.count().unwrap(), 0);
    }

    #[test]
    fn test_upsert_and_get() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let category_id = CategoryId::new();
        let period = test_period();

        let alloc =
            BudgetAllocation::with_budget(category_id, period.clone(), Money::from_cents(50000));

        repo.upsert(alloc).unwrap();

        let retrieved = repo.get(category_id, &period).unwrap().unwrap();
        assert_eq!(retrieved.budgeted.cents(), 50000);
    }

    #[test]
    fn test_get_or_default() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let category_id = CategoryId::new();
        let period = test_period();

        // Should return default (zero) if not found
        let alloc = repo.get_or_default(category_id, &period).unwrap();
        assert_eq!(alloc.budgeted.cents(), 0);

        // Now insert
        let alloc2 =
            BudgetAllocation::with_budget(category_id, period.clone(), Money::from_cents(100));
        repo.upsert(alloc2).unwrap();

        // Should return the inserted value
        let alloc3 = repo.get_or_default(category_id, &period).unwrap();
        assert_eq!(alloc3.budgeted.cents(), 100);
    }

    #[test]
    fn test_get_for_period() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let cat1 = CategoryId::new();
        let cat2 = CategoryId::new();
        let jan = BudgetPeriod::monthly(2025, 1);
        let feb = BudgetPeriod::monthly(2025, 2);

        repo.upsert(BudgetAllocation::with_budget(
            cat1,
            jan.clone(),
            Money::from_cents(100),
        ))
        .unwrap();
        repo.upsert(BudgetAllocation::with_budget(
            cat2,
            jan.clone(),
            Money::from_cents(200),
        ))
        .unwrap();
        repo.upsert(BudgetAllocation::with_budget(
            cat1,
            feb.clone(),
            Money::from_cents(300),
        ))
        .unwrap();

        let jan_allocs = repo.get_for_period(&jan).unwrap();
        assert_eq!(jan_allocs.len(), 2);

        let feb_allocs = repo.get_for_period(&feb).unwrap();
        assert_eq!(feb_allocs.len(), 1);
    }

    #[test]
    fn test_save_and_reload() {
        let (temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let category_id = CategoryId::new();
        let period = test_period();
        let alloc =
            BudgetAllocation::with_budget(category_id, period.clone(), Money::from_cents(50000));

        repo.upsert(alloc).unwrap();
        repo.save().unwrap();

        // Create new repo and load
        let path = temp_dir.path().join("budget.json");
        let repo2 = BudgetRepository::new(path);
        repo2.load().unwrap();

        let retrieved = repo2.get(category_id, &period).unwrap().unwrap();
        assert_eq!(retrieved.budgeted.cents(), 50000);
    }

    #[test]
    fn test_delete() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let category_id = CategoryId::new();
        let period = test_period();
        let alloc =
            BudgetAllocation::with_budget(category_id, period.clone(), Money::from_cents(100));

        repo.upsert(alloc).unwrap();
        assert_eq!(repo.count().unwrap(), 1);

        repo.delete(category_id, &period).unwrap();
        assert_eq!(repo.count().unwrap(), 0);
    }

    #[test]
    fn test_delete_for_category() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let cat1 = CategoryId::new();
        let cat2 = CategoryId::new();
        let jan = BudgetPeriod::monthly(2025, 1);
        let feb = BudgetPeriod::monthly(2025, 2);

        repo.upsert(BudgetAllocation::new(cat1, jan.clone()))
            .unwrap();
        repo.upsert(BudgetAllocation::new(cat1, feb.clone()))
            .unwrap();
        repo.upsert(BudgetAllocation::new(cat2, jan.clone()))
            .unwrap();

        assert_eq!(repo.count().unwrap(), 3);

        let deleted = repo.delete_for_category(cat1).unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(repo.count().unwrap(), 1);
    }
}
