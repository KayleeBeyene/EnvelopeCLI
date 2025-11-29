//! Income service
//!
//! Provides business logic for managing expected income per budget period.

use crate::audit::EntityType;
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{BudgetPeriod, IncomeExpectation, Money};
use crate::storage::Storage;

/// Service for income expectation management
pub struct IncomeService<'a> {
    storage: &'a Storage,
}

impl<'a> IncomeService<'a> {
    /// Create a new income service
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Set expected income for a period
    pub fn set_expected_income(
        &self,
        period: &BudgetPeriod,
        amount: Money,
        notes: Option<String>,
    ) -> EnvelopeResult<IncomeExpectation> {
        // Check if there's an existing expectation
        if let Some(existing) = self.storage.income.get_for_period(period) {
            let mut updated = existing.clone();
            let before = existing.clone();

            updated.set_expected_amount(amount);
            if let Some(n) = notes {
                updated.set_notes(n);
            }

            // Validate
            updated
                .validate()
                .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

            // Save
            self.storage.income.upsert(updated.clone())?;
            self.storage.income.save()?;

            // Audit update
            self.storage.log_update(
                EntityType::IncomeExpectation,
                updated.id.to_string(),
                Some(format!("Income for {}", period)),
                &before,
                &updated,
                Some(format!("{} -> {}", before.expected_amount, updated.expected_amount)),
            )?;

            Ok(updated)
        } else {
            let mut expectation = IncomeExpectation::new(period.clone(), amount);
            if let Some(n) = notes {
                expectation.set_notes(n);
            }

            // Validate
            expectation
                .validate()
                .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

            // Save
            self.storage.income.upsert(expectation.clone())?;
            self.storage.income.save()?;

            // Audit create
            self.storage.log_create(
                EntityType::IncomeExpectation,
                expectation.id.to_string(),
                Some(format!("Income for {}", period)),
                &expectation,
            )?;

            Ok(expectation)
        }
    }

    /// Get expected income amount for a period
    pub fn get_expected_income(&self, period: &BudgetPeriod) -> Option<Money> {
        self.storage
            .income
            .get_for_period(period)
            .map(|e| e.expected_amount)
    }

    /// Get the full income expectation for a period
    pub fn get_income_expectation(&self, period: &BudgetPeriod) -> Option<IncomeExpectation> {
        self.storage.income.get_for_period(period)
    }

    /// Delete income expectation for a period
    pub fn delete_expected_income(&self, period: &BudgetPeriod) -> EnvelopeResult<bool> {
        if let Some(removed) = self.storage.income.delete_for_period(period) {
            self.storage.income.save()?;

            // Audit delete
            self.storage.log_delete(
                EntityType::IncomeExpectation,
                removed.id.to_string(),
                Some(format!("Income for {}", period)),
                &removed,
            )?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get all income expectations
    pub fn get_all_expectations(&self) -> EnvelopeResult<Vec<IncomeExpectation>> {
        self.storage.income.get_all()
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
    fn test_set_expected_income() {
        let (_temp_dir, storage) = create_test_storage();
        let service = IncomeService::new(&storage);
        let period = BudgetPeriod::monthly(2025, 1);

        let expectation = service
            .set_expected_income(&period, Money::from_cents(500000), None)
            .unwrap();

        assert_eq!(expectation.expected_amount.cents(), 500000);
        assert_eq!(expectation.period, period);
    }

    #[test]
    fn test_update_expected_income() {
        let (_temp_dir, storage) = create_test_storage();
        let service = IncomeService::new(&storage);
        let period = BudgetPeriod::monthly(2025, 1);

        // Set initial
        service
            .set_expected_income(&period, Money::from_cents(500000), None)
            .unwrap();

        // Update
        let updated = service
            .set_expected_income(&period, Money::from_cents(600000), Some("Updated".to_string()))
            .unwrap();

        assert_eq!(updated.expected_amount.cents(), 600000);
        assert_eq!(updated.notes, "Updated");
    }

    #[test]
    fn test_get_expected_income() {
        let (_temp_dir, storage) = create_test_storage();
        let service = IncomeService::new(&storage);
        let period = BudgetPeriod::monthly(2025, 1);

        // No income set
        assert!(service.get_expected_income(&period).is_none());

        // Set income
        service
            .set_expected_income(&period, Money::from_cents(500000), None)
            .unwrap();

        // Now it should be Some
        let income = service.get_expected_income(&period).unwrap();
        assert_eq!(income.cents(), 500000);
    }

    #[test]
    fn test_delete_expected_income() {
        let (_temp_dir, storage) = create_test_storage();
        let service = IncomeService::new(&storage);
        let period = BudgetPeriod::monthly(2025, 1);

        // Set and delete
        service
            .set_expected_income(&period, Money::from_cents(500000), None)
            .unwrap();

        let deleted = service.delete_expected_income(&period).unwrap();
        assert!(deleted);

        // Should be gone
        assert!(service.get_expected_income(&period).is_none());

        // Deleting again should return false
        let deleted_again = service.delete_expected_income(&period).unwrap();
        assert!(!deleted_again);
    }

    #[test]
    fn test_negative_amount_rejected() {
        let (_temp_dir, storage) = create_test_storage();
        let service = IncomeService::new(&storage);
        let period = BudgetPeriod::monthly(2025, 1);

        let result = service.set_expected_income(&period, Money::from_cents(-100), None);
        assert!(result.is_err());
    }
}
