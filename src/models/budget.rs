//! Budget allocation model
//!
//! Tracks how much money is assigned to each category per budget period.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::ids::CategoryId;
use super::money::Money;
use super::period::BudgetPeriod;

/// A budget allocation for a specific category in a specific period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAllocation {
    /// The category this allocation is for
    pub category_id: CategoryId,

    /// The budget period
    pub period: BudgetPeriod,

    /// Amount budgeted/assigned to this category this period
    pub budgeted: Money,

    /// Amount carried over from the previous period (positive or negative)
    pub carryover: Money,

    /// Notes for this period's allocation
    #[serde(default)]
    pub notes: String,

    /// When this allocation was created
    pub created_at: DateTime<Utc>,

    /// When this allocation was last modified
    pub updated_at: DateTime<Utc>,
}

impl BudgetAllocation {
    /// Create a new budget allocation
    pub fn new(category_id: CategoryId, period: BudgetPeriod) -> Self {
        let now = Utc::now();
        Self {
            category_id,
            period,
            budgeted: Money::zero(),
            carryover: Money::zero(),
            notes: String::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create an allocation with an initial budget amount
    pub fn with_budget(category_id: CategoryId, period: BudgetPeriod, budgeted: Money) -> Self {
        let mut allocation = Self::new(category_id, period);
        allocation.budgeted = budgeted;
        allocation
    }

    /// Set the budgeted amount
    pub fn set_budgeted(&mut self, amount: Money) {
        self.budgeted = amount;
        self.updated_at = Utc::now();
    }

    /// Add to the budgeted amount
    pub fn add_budgeted(&mut self, amount: Money) {
        self.budgeted += amount;
        self.updated_at = Utc::now();
    }

    /// Set the carryover amount
    pub fn set_carryover(&mut self, amount: Money) {
        self.carryover = amount;
        self.updated_at = Utc::now();
    }

    /// Get the total available in this category (budgeted + carryover)
    /// Note: Activity (spending) must be subtracted by the caller who has transaction data
    pub fn total_budgeted(&self) -> Money {
        self.budgeted + self.carryover
    }

    /// Validate the allocation
    pub fn validate(&self) -> Result<(), BudgetValidationError> {
        // Budgeted amount cannot be negative
        if self.budgeted.is_negative() {
            return Err(BudgetValidationError::NegativeBudget);
        }

        Ok(())
    }
}

impl fmt::Display for BudgetAllocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} budgeted: {} (carryover: {})",
            self.period, self.budgeted, self.carryover
        )
    }
}

/// A summary of a category's budget status for a period
#[derive(Debug, Clone)]
pub struct CategoryBudgetSummary {
    /// Category ID
    pub category_id: CategoryId,

    /// The period
    pub period: BudgetPeriod,

    /// Amount budgeted this period
    pub budgeted: Money,

    /// Amount carried over from previous period
    pub carryover: Money,

    /// Activity (sum of transactions) - negative means spending
    pub activity: Money,

    /// Available = budgeted + carryover + activity
    pub available: Money,
}

impl CategoryBudgetSummary {
    /// Create a new summary
    pub fn new(
        category_id: CategoryId,
        period: BudgetPeriod,
        budgeted: Money,
        carryover: Money,
        activity: Money,
    ) -> Self {
        let available = budgeted + carryover + activity;
        Self {
            category_id,
            period,
            budgeted,
            carryover,
            activity,
            available,
        }
    }

    /// Create an empty summary for a category (all zeros)
    pub fn empty(category_id: CategoryId) -> Self {
        Self {
            category_id,
            period: BudgetPeriod::current_month(),
            budgeted: Money::zero(),
            carryover: Money::zero(),
            activity: Money::zero(),
            available: Money::zero(),
        }
    }

    /// Create from an allocation and activity amount
    pub fn from_allocation(allocation: &BudgetAllocation, activity: Money) -> Self {
        Self::new(
            allocation.category_id,
            allocation.period.clone(),
            allocation.budgeted,
            allocation.carryover,
            activity,
        )
    }

    /// Check if this category is overspent (available is negative)
    pub fn is_overspent(&self) -> bool {
        self.available.is_negative()
    }

    /// Check if this category is underfunded (budgeted < goal, if goal is set)
    pub fn is_underfunded(&self, goal: Option<Money>) -> bool {
        if let Some(goal_amount) = goal {
            self.budgeted < goal_amount
        } else {
            false
        }
    }

    /// Get the amount that would roll over to next period
    pub fn rollover_amount(&self) -> Money {
        self.available
    }
}

impl fmt::Display for CategoryBudgetSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Budgeted: {} | Activity: {} | Available: {}",
            self.budgeted, self.activity, self.available
        )
    }
}

/// Validation errors for budget allocations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetValidationError {
    NegativeBudget,
}

impl fmt::Display for BudgetValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeBudget => write!(f, "Budget amount cannot be negative"),
        }
    }
}

impl std::error::Error for BudgetValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_category_id() -> CategoryId {
        CategoryId::new()
    }

    fn test_period() -> BudgetPeriod {
        BudgetPeriod::monthly(2025, 1)
    }

    #[test]
    fn test_new_allocation() {
        let category_id = test_category_id();
        let period = test_period();
        let allocation = BudgetAllocation::new(category_id, period.clone());

        assert_eq!(allocation.category_id, category_id);
        assert_eq!(allocation.period, period);
        assert_eq!(allocation.budgeted, Money::zero());
        assert_eq!(allocation.carryover, Money::zero());
    }

    #[test]
    fn test_with_budget() {
        let category_id = test_category_id();
        let period = test_period();
        let allocation =
            BudgetAllocation::with_budget(category_id, period, Money::from_cents(50000));

        assert_eq!(allocation.budgeted.cents(), 50000);
    }

    #[test]
    fn test_total_budgeted() {
        let category_id = test_category_id();
        let period = test_period();
        let mut allocation = BudgetAllocation::new(category_id, period);
        allocation.budgeted = Money::from_cents(50000);
        allocation.carryover = Money::from_cents(10000);

        assert_eq!(allocation.total_budgeted().cents(), 60000);
    }

    #[test]
    fn test_negative_carryover() {
        let category_id = test_category_id();
        let period = test_period();
        let mut allocation = BudgetAllocation::new(category_id, period);
        allocation.budgeted = Money::from_cents(50000);
        allocation.carryover = Money::from_cents(-20000); // Overspent last period

        assert_eq!(allocation.total_budgeted().cents(), 30000);
    }

    #[test]
    fn test_validation() {
        let category_id = test_category_id();
        let period = test_period();
        let mut allocation = BudgetAllocation::new(category_id, period);

        allocation.budgeted = Money::from_cents(50000);
        assert!(allocation.validate().is_ok());

        allocation.budgeted = Money::from_cents(-100);
        assert_eq!(
            allocation.validate(),
            Err(BudgetValidationError::NegativeBudget)
        );
    }

    #[test]
    fn test_category_summary() {
        let category_id = test_category_id();
        let period = test_period();
        let budgeted = Money::from_cents(50000);
        let carryover = Money::from_cents(10000);
        let activity = Money::from_cents(-30000); // Spent $300

        let summary = CategoryBudgetSummary::new(category_id, period, budgeted, carryover, activity);

        assert_eq!(summary.budgeted.cents(), 50000);
        assert_eq!(summary.carryover.cents(), 10000);
        assert_eq!(summary.activity.cents(), -30000);
        assert_eq!(summary.available.cents(), 30000); // 500 + 100 - 300 = 300
        assert!(!summary.is_overspent());
    }

    #[test]
    fn test_overspent_summary() {
        let category_id = test_category_id();
        let period = test_period();
        let budgeted = Money::from_cents(50000);
        let carryover = Money::zero();
        let activity = Money::from_cents(-60000); // Overspent by $100

        let summary = CategoryBudgetSummary::new(category_id, period, budgeted, carryover, activity);

        assert!(summary.is_overspent());
        assert_eq!(summary.available.cents(), -10000);
        assert_eq!(summary.rollover_amount().cents(), -10000);
    }

    #[test]
    fn test_serialization() {
        let category_id = test_category_id();
        let period = test_period();
        let allocation =
            BudgetAllocation::with_budget(category_id, period, Money::from_cents(50000));

        let json = serde_json::to_string(&allocation).unwrap();
        let deserialized: BudgetAllocation = serde_json::from_str(&json).unwrap();
        assert_eq!(allocation.category_id, deserialized.category_id);
        assert_eq!(allocation.budgeted, deserialized.budgeted);
    }
}
