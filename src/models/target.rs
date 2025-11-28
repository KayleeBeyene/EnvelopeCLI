//! Budget target model
//!
//! Tracks recurring budget targets for categories, supporting various cadences
//! like YNAB: weekly, monthly, yearly, custom intervals, and by-date goals.

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::ids::CategoryId;
use super::money::Money;
use super::period::BudgetPeriod;

/// Unique identifier for a budget target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BudgetTargetId(uuid::Uuid);

impl BudgetTargetId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(uuid::Uuid::parse_str(s)?))
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl Default for BudgetTargetId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BudgetTargetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tgt-{}", &self.0.to_string()[..8])
    }
}

/// The cadence/frequency at which a budget target repeats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum TargetCadence {
    Weekly,
    Monthly,
    Yearly,
    Custom { days: u32 },
    ByDate { target_date: NaiveDate },
}

impl TargetCadence {
    pub fn weekly() -> Self {
        Self::Weekly
    }

    pub fn monthly() -> Self {
        Self::Monthly
    }

    pub fn yearly() -> Self {
        Self::Yearly
    }

    pub fn custom(days: u32) -> Self {
        Self::Custom { days }
    }

    pub fn by_date(target_date: NaiveDate) -> Self {
        Self::ByDate { target_date }
    }

    pub fn description(&self) -> String {
        match self {
            Self::Weekly => "Weekly".to_string(),
            Self::Monthly => "Monthly".to_string(),
            Self::Yearly => "Yearly".to_string(),
            Self::Custom { days } => format!("Every {} days", days),
            Self::ByDate { target_date } => format!("By {}", target_date.format("%Y-%m-%d")),
        }
    }
}

impl fmt::Display for TargetCadence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// A budget target for a category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetTarget {
    pub id: BudgetTargetId,
    pub category_id: CategoryId,
    pub amount: Money,
    pub cadence: TargetCadence,
    #[serde(default)]
    pub notes: String,
    #[serde(default = "default_active")]
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_active() -> bool {
    true
}

impl BudgetTarget {
    pub fn new(category_id: CategoryId, amount: Money, cadence: TargetCadence) -> Self {
        let now = Utc::now();
        Self {
            id: BudgetTargetId::new(),
            category_id,
            amount,
            cadence,
            notes: String::new(),
            active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn monthly(category_id: CategoryId, amount: Money) -> Self {
        Self::new(category_id, amount, TargetCadence::Monthly)
    }

    pub fn weekly(category_id: CategoryId, amount: Money) -> Self {
        Self::new(category_id, amount, TargetCadence::Weekly)
    }

    pub fn yearly(category_id: CategoryId, amount: Money) -> Self {
        Self::new(category_id, amount, TargetCadence::Yearly)
    }

    pub fn calculate_for_period(&self, period: &BudgetPeriod) -> Money {
        if !self.active {
            return Money::zero();
        }

        match &self.cadence {
            TargetCadence::Weekly => self.calculate_weekly_for_period(period),
            TargetCadence::Monthly => self.calculate_monthly_for_period(period),
            TargetCadence::Yearly => self.calculate_yearly_for_period(period),
            TargetCadence::Custom { days } => self.calculate_custom_for_period(period, *days),
            TargetCadence::ByDate { target_date } => {
                self.calculate_by_date_for_period(period, *target_date)
            }
        }
    }

    fn calculate_weekly_for_period(&self, period: &BudgetPeriod) -> Money {
        match period {
            BudgetPeriod::Weekly { .. } => self.amount,
            BudgetPeriod::Monthly { year, month } => {
                let start = NaiveDate::from_ymd_opt(*year, *month, 1).unwrap();
                let end = if *month == 12 {
                    NaiveDate::from_ymd_opt(*year + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(*year, *month + 1, 1).unwrap()
                };
                let days = (end - start).num_days() as f64;
                let weeks = days / 7.0;
                Money::from_cents((self.amount.cents() as f64 * weeks).round() as i64)
            }
            BudgetPeriod::BiWeekly { .. } => Money::from_cents(self.amount.cents() * 2),
            BudgetPeriod::Custom { start, end } => {
                let days = (*end - *start).num_days() as f64 + 1.0;
                let weeks = days / 7.0;
                Money::from_cents((self.amount.cents() as f64 * weeks).round() as i64)
            }
        }
    }

    fn calculate_monthly_for_period(&self, period: &BudgetPeriod) -> Money {
        match period {
            BudgetPeriod::Monthly { .. } => self.amount,
            BudgetPeriod::Weekly { .. } => {
                Money::from_cents((self.amount.cents() as f64 / 4.33).round() as i64)
            }
            BudgetPeriod::BiWeekly { .. } => Money::from_cents(self.amount.cents() / 2),
            BudgetPeriod::Custom { start, end } => {
                let days = (*end - *start).num_days() as f64 + 1.0;
                Money::from_cents((self.amount.cents() as f64 * days / 30.0).round() as i64)
            }
        }
    }

    fn calculate_yearly_for_period(&self, period: &BudgetPeriod) -> Money {
        match period {
            BudgetPeriod::Monthly { .. } => Money::from_cents(self.amount.cents() / 12),
            BudgetPeriod::Weekly { .. } => {
                Money::from_cents((self.amount.cents() as f64 / 52.0).round() as i64)
            }
            BudgetPeriod::BiWeekly { .. } => {
                Money::from_cents((self.amount.cents() as f64 / 26.0).round() as i64)
            }
            BudgetPeriod::Custom { start, end } => {
                let days = (*end - *start).num_days() as f64 + 1.0;
                Money::from_cents((self.amount.cents() as f64 * days / 365.0).round() as i64)
            }
        }
    }

    fn calculate_custom_for_period(&self, period: &BudgetPeriod, interval_days: u32) -> Money {
        let period_days = (period.end_date() - period.start_date()).num_days() as f64 + 1.0;
        let intervals = period_days / interval_days as f64;
        Money::from_cents((self.amount.cents() as f64 * intervals).round() as i64)
    }

    fn calculate_by_date_for_period(&self, period: &BudgetPeriod, target_date: NaiveDate) -> Money {
        let period_start = period.start_date();
        let period_end = period.end_date();

        if target_date < period_start {
            return Money::zero();
        }

        if target_date <= period_end {
            return self.amount;
        }

        let months_remaining = self.months_between(period_start, target_date);
        if months_remaining <= 0 {
            return self.amount;
        }

        Money::from_cents((self.amount.cents() as f64 / months_remaining as f64).ceil() as i64)
    }

    fn months_between(&self, start: NaiveDate, end: NaiveDate) -> i32 {
        let years = end.year() - start.year();
        let months = end.month() as i32 - start.month() as i32;
        years * 12 + months
    }

    pub fn set_amount(&mut self, amount: Money) {
        self.amount = amount;
        self.updated_at = Utc::now();
    }

    pub fn set_cadence(&mut self, cadence: TargetCadence) {
        self.cadence = cadence;
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.updated_at = Utc::now();
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated_at = Utc::now();
    }

    pub fn validate(&self) -> Result<(), TargetValidationError> {
        if self.amount.is_negative() {
            return Err(TargetValidationError::NegativeAmount);
        }

        if self.amount.is_zero() {
            return Err(TargetValidationError::ZeroAmount);
        }

        if let TargetCadence::Custom { days } = self.cadence {
            if days == 0 {
                return Err(TargetValidationError::InvalidCustomInterval);
            }
        }

        Ok(())
    }
}

impl fmt::Display for BudgetTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.cadence)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetValidationError {
    NegativeAmount,
    ZeroAmount,
    InvalidCustomInterval,
}

impl fmt::Display for TargetValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeAmount => write!(f, "Target amount cannot be negative"),
            Self::ZeroAmount => write!(f, "Target amount cannot be zero"),
            Self::InvalidCustomInterval => write!(f, "Custom interval must be at least 1 day"),
        }
    }
}

impl std::error::Error for TargetValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_category_id() -> CategoryId {
        CategoryId::new()
    }

    #[test]
    fn test_new_target() {
        let category_id = test_category_id();
        let target = BudgetTarget::monthly(category_id, Money::from_cents(50000));

        assert_eq!(target.category_id, category_id);
        assert_eq!(target.amount.cents(), 50000);
        assert!(matches!(target.cadence, TargetCadence::Monthly));
        assert!(target.active);
    }

    #[test]
    fn test_monthly_target_for_monthly_period() {
        let target = BudgetTarget::monthly(test_category_id(), Money::from_cents(50000));
        let period = BudgetPeriod::monthly(2025, 1);

        let suggested = target.calculate_for_period(&period);
        assert_eq!(suggested.cents(), 50000);
    }

    #[test]
    fn test_yearly_target_for_monthly_period() {
        let target = BudgetTarget::yearly(test_category_id(), Money::from_cents(120000));
        let period = BudgetPeriod::monthly(2025, 1);

        let suggested = target.calculate_for_period(&period);
        assert_eq!(suggested.cents(), 10000);
    }

    #[test]
    fn test_validation() {
        let target = BudgetTarget::monthly(test_category_id(), Money::from_cents(50000));
        assert!(target.validate().is_ok());

        let negative_target = BudgetTarget::monthly(test_category_id(), Money::from_cents(-100));
        assert_eq!(
            negative_target.validate(),
            Err(TargetValidationError::NegativeAmount)
        );

        let zero_target = BudgetTarget::monthly(test_category_id(), Money::zero());
        assert_eq!(
            zero_target.validate(),
            Err(TargetValidationError::ZeroAmount)
        );
    }

    #[test]
    fn test_serialization() {
        let target = BudgetTarget::monthly(test_category_id(), Money::from_cents(50000));
        let json = serde_json::to_string(&target).unwrap();
        let deserialized: BudgetTarget = serde_json::from_str(&json).unwrap();

        assert_eq!(target.id, deserialized.id);
        assert_eq!(target.amount, deserialized.amount);
        assert_eq!(target.cadence, deserialized.cadence);
    }
}
