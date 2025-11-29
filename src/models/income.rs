//! Income expectation model
//!
//! Tracks expected income per budget period, allowing users to see
//! when they're budgeting more than they expect to earn.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids::IncomeId;
use super::money::Money;
use super::period::BudgetPeriod;

/// Validation errors for income expectations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncomeValidationError {
    NegativeAmount,
}

impl std::fmt::Display for IncomeValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NegativeAmount => write!(f, "Expected income cannot be negative"),
        }
    }
}

impl std::error::Error for IncomeValidationError {}

/// Expected income for a budget period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeExpectation {
    pub id: IncomeId,
    pub period: BudgetPeriod,
    pub expected_amount: Money,
    #[serde(default)]
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IncomeExpectation {
    /// Create a new income expectation
    pub fn new(period: BudgetPeriod, expected_amount: Money) -> Self {
        let now = Utc::now();
        Self {
            id: IncomeId::new(),
            period,
            expected_amount,
            notes: String::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the expected amount
    pub fn set_expected_amount(&mut self, amount: Money) {
        self.expected_amount = amount;
        self.updated_at = Utc::now();
    }

    /// Set notes
    pub fn set_notes(&mut self, notes: impl Into<String>) {
        self.notes = notes.into();
        self.updated_at = Utc::now();
    }

    /// Validate the income expectation
    pub fn validate(&self) -> Result<(), IncomeValidationError> {
        if self.expected_amount.is_negative() {
            return Err(IncomeValidationError::NegativeAmount);
        }
        Ok(())
    }

    /// Check if a budgeted amount exceeds expected income
    pub fn is_over_budget(&self, total_budgeted: Money) -> bool {
        total_budgeted > self.expected_amount
    }

    /// Get the difference between expected income and budgeted amount
    /// Positive = under budget (good), Negative = over budget (warning)
    pub fn budget_difference(&self, total_budgeted: Money) -> Money {
        self.expected_amount - total_budgeted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_income_expectation() {
        let period = BudgetPeriod::monthly(2025, 1);
        let income = IncomeExpectation::new(period.clone(), Money::from_cents(500000));

        assert_eq!(income.period, period);
        assert_eq!(income.expected_amount.cents(), 500000);
        assert!(income.notes.is_empty());
    }

    #[test]
    fn test_validation_negative_amount() {
        let period = BudgetPeriod::monthly(2025, 1);
        let income = IncomeExpectation::new(period, Money::from_cents(-100));

        assert!(matches!(
            income.validate(),
            Err(IncomeValidationError::NegativeAmount)
        ));
    }

    #[test]
    fn test_over_budget_detection() {
        let period = BudgetPeriod::monthly(2025, 1);
        let income = IncomeExpectation::new(period, Money::from_cents(500000)); // $5000

        // Under budget
        assert!(!income.is_over_budget(Money::from_cents(400000))); // $4000

        // Exactly at budget
        assert!(!income.is_over_budget(Money::from_cents(500000))); // $5000

        // Over budget
        assert!(income.is_over_budget(Money::from_cents(600000))); // $6000
    }

    #[test]
    fn test_budget_difference() {
        let period = BudgetPeriod::monthly(2025, 1);
        let income = IncomeExpectation::new(period, Money::from_cents(500000)); // $5000

        // Under budget by $1000
        let diff = income.budget_difference(Money::from_cents(400000));
        assert_eq!(diff.cents(), 100000);

        // Over budget by $1000
        let diff = income.budget_difference(Money::from_cents(600000));
        assert_eq!(diff.cents(), -100000);
    }

    #[test]
    fn test_set_expected_amount() {
        let period = BudgetPeriod::monthly(2025, 1);
        let mut income = IncomeExpectation::new(period, Money::from_cents(500000));
        let original_updated = income.updated_at;

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        income.set_expected_amount(Money::from_cents(600000));
        assert_eq!(income.expected_amount.cents(), 600000);
        assert!(income.updated_at >= original_updated);
    }

    #[test]
    fn test_set_notes() {
        let period = BudgetPeriod::monthly(2025, 1);
        let mut income = IncomeExpectation::new(period, Money::from_cents(500000));

        income.set_notes("Includes bonus");
        assert_eq!(income.notes, "Includes bonus");
    }

    #[test]
    fn test_serialization() {
        let period = BudgetPeriod::monthly(2025, 1);
        let income = IncomeExpectation::new(period, Money::from_cents(500000));

        let json = serde_json::to_string(&income).unwrap();
        let deserialized: IncomeExpectation = serde_json::from_str(&json).unwrap();

        assert_eq!(income.id, deserialized.id);
        assert_eq!(income.period, deserialized.period);
        assert_eq!(income.expected_amount, deserialized.expected_amount);
    }
}
