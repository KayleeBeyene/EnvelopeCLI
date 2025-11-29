//! Income expectations repository
//!
//! Handles persistence of income expectations to JSON files.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::error::EnvelopeError;
use crate::models::{BudgetPeriod, IncomeExpectation, IncomeId};

use super::file_io::{read_json, write_json_atomic};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct IncomeData {
    #[serde(default)]
    expectations: Vec<IncomeExpectation>,
}

/// Repository for income expectations
pub struct IncomeRepository {
    path: PathBuf,
    expectations: RwLock<HashMap<BudgetPeriod, IncomeExpectation>>,
}

impl IncomeRepository {
    /// Create a new repository
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            expectations: RwLock::new(HashMap::new()),
        }
    }

    /// Load expectations from disk
    pub fn load(&self) -> Result<(), EnvelopeError> {
        let file_data: IncomeData = read_json(&self.path)?;

        let mut expectations = self
            .expectations
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        expectations.clear();
        for expectation in file_data.expectations {
            expectations.insert(expectation.period.clone(), expectation);
        }

        Ok(())
    }

    /// Save expectations to disk
    pub fn save(&self) -> Result<(), EnvelopeError> {
        let expectations = self
            .expectations
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let mut list: Vec<_> = expectations.values().cloned().collect();
        list.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        let file_data = IncomeData { expectations: list };

        write_json_atomic(&self.path, &file_data)
    }

    /// Get income expectation for a period
    pub fn get_for_period(&self, period: &BudgetPeriod) -> Option<IncomeExpectation> {
        let expectations = self.expectations.read().ok()?;
        expectations.get(period).cloned()
    }

    /// Get income expectation by ID
    pub fn get(&self, id: IncomeId) -> Result<Option<IncomeExpectation>, EnvelopeError> {
        let expectations = self
            .expectations
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        Ok(expectations.values().find(|e| e.id == id).cloned())
    }

    /// Upsert an income expectation (insert or update)
    pub fn upsert(&self, expectation: IncomeExpectation) -> Result<(), EnvelopeError> {
        let mut expectations = self
            .expectations
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        expectations.insert(expectation.period.clone(), expectation);
        Ok(())
    }

    /// Delete income expectation for a period
    pub fn delete_for_period(&self, period: &BudgetPeriod) -> Option<IncomeExpectation> {
        let mut expectations = self.expectations.write().ok()?;
        expectations.remove(period)
    }

    /// Get all income expectations
    pub fn get_all(&self) -> Result<Vec<IncomeExpectation>, EnvelopeError> {
        let expectations = self
            .expectations
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let mut list: Vec<_> = expectations.values().cloned().collect();
        list.sort_by(|a, b| a.period.cmp(&b.period));
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Money;
    use tempfile::TempDir;

    #[test]
    fn test_upsert_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("income.json");
        let repo = IncomeRepository::new(path);

        let period = BudgetPeriod::monthly(2025, 1);
        let expectation = IncomeExpectation::new(period.clone(), Money::from_cents(500000));

        repo.upsert(expectation).unwrap();

        let retrieved = repo.get_for_period(&period).unwrap();
        assert_eq!(retrieved.expected_amount.cents(), 500000);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("income.json");

        // Save
        {
            let repo = IncomeRepository::new(path.clone());
            let period = BudgetPeriod::monthly(2025, 1);
            let expectation = IncomeExpectation::new(period, Money::from_cents(500000));
            repo.upsert(expectation).unwrap();
            repo.save().unwrap();
        }

        // Load
        {
            let repo = IncomeRepository::new(path);
            repo.load().unwrap();
            let period = BudgetPeriod::monthly(2025, 1);
            let retrieved = repo.get_for_period(&period).unwrap();
            assert_eq!(retrieved.expected_amount.cents(), 500000);
        }
    }

    #[test]
    fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("income.json");
        let repo = IncomeRepository::new(path);

        let period = BudgetPeriod::monthly(2025, 1);
        let expectation = IncomeExpectation::new(period.clone(), Money::from_cents(500000));

        repo.upsert(expectation).unwrap();
        assert!(repo.get_for_period(&period).is_some());

        repo.delete_for_period(&period);
        assert!(repo.get_for_period(&period).is_none());
    }

    #[test]
    fn test_get_all() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("income.json");
        let repo = IncomeRepository::new(path);

        let period1 = BudgetPeriod::monthly(2025, 1);
        let period2 = BudgetPeriod::monthly(2025, 2);

        repo.upsert(IncomeExpectation::new(period1, Money::from_cents(500000)))
            .unwrap();
        repo.upsert(IncomeExpectation::new(period2, Money::from_cents(550000)))
            .unwrap();

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 2);
    }
}
