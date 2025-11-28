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

    // ============================================
    // Edge Case Tests for Period Calculations
    // ============================================

    #[test]
    fn test_weekly_target_for_leap_year_february() {
        // February 2024 has 29 days (leap year)
        let target = BudgetTarget::weekly(test_category_id(), Money::from_cents(7000)); // $70/week
        let period = BudgetPeriod::monthly(2024, 2);

        let suggested = target.calculate_for_period(&period);
        // 29 days / 7 days = ~4.14 weeks
        // 7000 cents * 4.14 = ~29000 cents
        let weeks: f64 = 29.0 / 7.0;
        let expected = (7000.0_f64 * weeks).round() as i64;
        assert_eq!(suggested.cents(), expected);
    }

    #[test]
    fn test_weekly_target_for_non_leap_year_february() {
        // February 2025 has 28 days (non-leap year)
        let target = BudgetTarget::weekly(test_category_id(), Money::from_cents(7000)); // $70/week
        let period = BudgetPeriod::monthly(2025, 2);

        let suggested = target.calculate_for_period(&period);
        // 28 days / 7 days = 4 weeks exactly
        let weeks: f64 = 28.0 / 7.0;
        let expected = (7000.0_f64 * weeks).round() as i64;
        assert_eq!(suggested.cents(), expected);
    }

    #[test]
    fn test_weekly_target_for_31_day_month() {
        // January has 31 days
        let target = BudgetTarget::weekly(test_category_id(), Money::from_cents(7000));
        let period = BudgetPeriod::monthly(2025, 1);

        let suggested = target.calculate_for_period(&period);
        let weeks: f64 = 31.0 / 7.0;
        let expected = (7000.0_f64 * weeks).round() as i64;
        assert_eq!(suggested.cents(), expected);
    }

    #[test]
    fn test_weekly_target_for_30_day_month() {
        // April has 30 days
        let target = BudgetTarget::weekly(test_category_id(), Money::from_cents(7000));
        let period = BudgetPeriod::monthly(2025, 4);

        let suggested = target.calculate_for_period(&period);
        let weeks: f64 = 30.0 / 7.0;
        let expected = (7000.0_f64 * weeks).round() as i64;
        assert_eq!(suggested.cents(), expected);
    }

    #[test]
    fn test_monthly_target_for_weekly_period() {
        let target = BudgetTarget::monthly(test_category_id(), Money::from_cents(43300)); // ~$433/month
        let period = BudgetPeriod::weekly(2025, 1);

        let suggested = target.calculate_for_period(&period);
        // Monthly / 4.33 = weekly amount
        let expected = (43300.0_f64 / 4.33_f64).round() as i64;
        assert_eq!(suggested.cents(), expected);
    }

    #[test]
    fn test_monthly_target_for_biweekly_period() {
        let target = BudgetTarget::monthly(test_category_id(), Money::from_cents(100000)); // $1000/month
        let period = BudgetPeriod::bi_weekly(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());

        let suggested = target.calculate_for_period(&period);
        // Monthly / 2 = bi-weekly amount
        assert_eq!(suggested.cents(), 50000); // $500
    }

    #[test]
    fn test_yearly_target_for_weekly_period() {
        let target = BudgetTarget::yearly(test_category_id(), Money::from_cents(5200000)); // $52,000/year
        let period = BudgetPeriod::weekly(2025, 1);

        let suggested = target.calculate_for_period(&period);
        // $52,000 / 52 weeks = $1000/week
        let expected = (5200000.0_f64 / 52.0_f64).round() as i64;
        assert_eq!(suggested.cents(), expected);
    }

    #[test]
    fn test_yearly_target_for_biweekly_period() {
        let target = BudgetTarget::yearly(test_category_id(), Money::from_cents(2600000)); // $26,000/year
        let period = BudgetPeriod::bi_weekly(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());

        let suggested = target.calculate_for_period(&period);
        // $26,000 / 26 bi-weekly periods = $1000 per bi-week
        let expected = (2600000.0_f64 / 26.0_f64).round() as i64;
        assert_eq!(suggested.cents(), expected);
    }

    // ============================================
    // Custom Interval Tests
    // ============================================

    #[test]
    fn test_custom_interval_for_monthly_period() {
        // Target resets every 14 days with $100 target
        let target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(10000),
            TargetCadence::custom(14),
        );
        let period = BudgetPeriod::monthly(2025, 1); // 31 days

        let suggested = target.calculate_for_period(&period);
        // 31 days / 14 days = ~2.21 intervals
        // 10000 * 2.21 = ~22143 cents
        let intervals: f64 = 31.0 / 14.0;
        let expected = (10000.0_f64 * intervals).round() as i64;
        assert_eq!(suggested.cents(), expected);
    }

    #[test]
    fn test_custom_interval_for_weekly_period() {
        // Target resets every 3 days with $30 target
        let target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(3000),
            TargetCadence::custom(3),
        );
        let period = BudgetPeriod::weekly(2025, 1); // 7 days

        let suggested = target.calculate_for_period(&period);
        // Week is 7 days (end - start + 1), so 7/3 = ~2.33 intervals
        // But the formula uses (end - start).num_days() + 1 = 7
        let period_days: f64 = 7.0; // Weekly period is 7 days
        let intervals: f64 = period_days / 3.0;
        let expected = (3000.0_f64 * intervals).round() as i64;
        assert_eq!(suggested.cents(), expected);
    }

    #[test]
    fn test_custom_interval_one_day() {
        // Daily target
        let target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(1000),
            TargetCadence::custom(1),
        );
        let period = BudgetPeriod::monthly(2025, 1); // 31 days

        let suggested = target.calculate_for_period(&period);
        // 31 intervals at $10 each = $310
        assert_eq!(suggested.cents(), 31000);
    }

    #[test]
    fn test_custom_interval_validation() {
        let target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(10000),
            TargetCadence::custom(0), // Invalid: 0 days
        );

        assert_eq!(
            target.validate(),
            Err(TargetValidationError::InvalidCustomInterval)
        );
    }

    // ============================================
    // ByDate Cadence Tests
    // ============================================

    #[test]
    fn test_by_date_target_date_in_current_period() {
        // Target date is within the current period - should return full amount
        let target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(100000), // $1000
            TargetCadence::by_date(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
        );
        let period = BudgetPeriod::monthly(2025, 1);

        let suggested = target.calculate_for_period(&period);
        assert_eq!(suggested.cents(), 100000); // Full amount needed
    }

    #[test]
    fn test_by_date_target_date_passed() {
        // Target date has already passed - should return zero
        let target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(100000),
            TargetCadence::by_date(NaiveDate::from_ymd_opt(2024, 12, 15).unwrap()),
        );
        let period = BudgetPeriod::monthly(2025, 1);

        let suggested = target.calculate_for_period(&period);
        assert_eq!(suggested.cents(), 0);
    }

    #[test]
    fn test_by_date_six_months_away() {
        // Target is 6 months away
        let target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(600000), // $6000
            TargetCadence::by_date(NaiveDate::from_ymd_opt(2025, 7, 1).unwrap()),
        );
        let period = BudgetPeriod::monthly(2025, 1);

        let suggested = target.calculate_for_period(&period);
        // 6 months away = $6000 / 6 = $1000 per month
        // Using ceil, so 600000 / 6 = 100000
        assert_eq!(suggested.cents(), 100000);
    }

    #[test]
    fn test_by_date_twelve_months_away() {
        // Target is 12 months away
        let target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(1200000), // $12,000
            TargetCadence::by_date(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        );
        let period = BudgetPeriod::monthly(2025, 1);

        let suggested = target.calculate_for_period(&period);
        // 12 months away = $12,000 / 12 = $1000 per month
        assert_eq!(suggested.cents(), 100000);
    }

    #[test]
    fn test_by_date_one_month_away() {
        // Target is next month
        let target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(50000), // $500
            TargetCadence::by_date(NaiveDate::from_ymd_opt(2025, 2, 15).unwrap()),
        );
        let period = BudgetPeriod::monthly(2025, 1);

        let suggested = target.calculate_for_period(&period);
        // 1 month away = full amount needed this month
        assert_eq!(suggested.cents(), 50000);
    }

    #[test]
    fn test_by_date_uneven_distribution() {
        // $1000 over 3 months = $333.34 per month (rounded up)
        let target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(100000), // $1000
            TargetCadence::by_date(NaiveDate::from_ymd_opt(2025, 4, 1).unwrap()),
        );
        let period = BudgetPeriod::monthly(2025, 1);

        let suggested = target.calculate_for_period(&period);
        // 3 months away, using ceil: ceil(100000 / 3) = 33334
        let expected = (100000.0_f64 / 3.0_f64).ceil() as i64;
        assert_eq!(suggested.cents(), expected);
    }

    #[test]
    fn test_by_date_target_at_period_end() {
        // Target date is exactly at the end of the period
        let target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(100000),
            TargetCadence::by_date(NaiveDate::from_ymd_opt(2025, 1, 31).unwrap()),
        );
        let period = BudgetPeriod::monthly(2025, 1);

        let suggested = target.calculate_for_period(&period);
        assert_eq!(suggested.cents(), 100000); // Full amount needed
    }

    // ============================================
    // Inactive Target Tests
    // ============================================

    #[test]
    fn test_inactive_target_returns_zero() {
        let mut target = BudgetTarget::monthly(test_category_id(), Money::from_cents(50000));
        target.deactivate();

        let period = BudgetPeriod::monthly(2025, 1);
        let suggested = target.calculate_for_period(&period);

        assert_eq!(suggested.cents(), 0);
        assert!(!target.active);
    }

    #[test]
    fn test_reactivated_target() {
        let mut target = BudgetTarget::monthly(test_category_id(), Money::from_cents(50000));
        target.deactivate();
        target.activate();

        let period = BudgetPeriod::monthly(2025, 1);
        let suggested = target.calculate_for_period(&period);

        assert_eq!(suggested.cents(), 50000);
        assert!(target.active);
    }

    #[test]
    fn test_inactive_weekly_target() {
        let mut target = BudgetTarget::weekly(test_category_id(), Money::from_cents(7000));
        target.deactivate();

        let period = BudgetPeriod::weekly(2025, 1);
        let suggested = target.calculate_for_period(&period);

        assert_eq!(suggested.cents(), 0);
    }

    #[test]
    fn test_inactive_yearly_target() {
        let mut target = BudgetTarget::yearly(test_category_id(), Money::from_cents(120000));
        target.deactivate();

        let period = BudgetPeriod::monthly(2025, 1);
        let suggested = target.calculate_for_period(&period);

        assert_eq!(suggested.cents(), 0);
    }

    #[test]
    fn test_inactive_by_date_target() {
        let mut target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(100000),
            TargetCadence::by_date(NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()),
        );
        target.deactivate();

        let period = BudgetPeriod::monthly(2025, 1);
        let suggested = target.calculate_for_period(&period);

        assert_eq!(suggested.cents(), 0);
    }

    // ============================================
    // Custom Period Tests
    // ============================================

    #[test]
    fn test_weekly_target_for_custom_period() {
        let target = BudgetTarget::weekly(test_category_id(), Money::from_cents(7000));
        let period = BudgetPeriod::custom(
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 21).unwrap(),
        ); // 21 days

        let suggested = target.calculate_for_period(&period);
        // 21 days / 7 = 3 weeks
        let weeks: f64 = 21.0 / 7.0;
        let expected = (7000.0_f64 * weeks).round() as i64;
        assert_eq!(suggested.cents(), expected);
    }

    #[test]
    fn test_monthly_target_for_custom_period() {
        let target = BudgetTarget::monthly(test_category_id(), Money::from_cents(30000)); // $300/month
        let period = BudgetPeriod::custom(
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
        ); // 15 days

        let suggested = target.calculate_for_period(&period);
        // 15 days out of ~30 = half the monthly amount
        let days: f64 = 15.0;
        let expected = (30000.0_f64 * days / 30.0_f64).round() as i64;
        assert_eq!(suggested.cents(), expected);
    }

    #[test]
    fn test_yearly_target_for_custom_period() {
        let target = BudgetTarget::yearly(test_category_id(), Money::from_cents(3650000)); // $36,500/year
        let period = BudgetPeriod::custom(
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
        ); // 10 days

        let suggested = target.calculate_for_period(&period);
        // 10 days out of 365 = 10/365 of yearly
        let days: f64 = 10.0;
        let expected = (3650000.0_f64 * days / 365.0_f64).round() as i64;
        assert_eq!(suggested.cents(), expected);
    }

    // ============================================
    // Modification Tests
    // ============================================

    #[test]
    fn test_set_amount_updates_timestamp() {
        let mut target = BudgetTarget::monthly(test_category_id(), Money::from_cents(50000));
        let original_updated_at = target.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        target.set_amount(Money::from_cents(75000));

        assert_eq!(target.amount.cents(), 75000);
        assert!(target.updated_at > original_updated_at);
    }

    #[test]
    fn test_set_cadence_updates_timestamp() {
        let mut target = BudgetTarget::monthly(test_category_id(), Money::from_cents(50000));
        let original_updated_at = target.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        target.set_cadence(TargetCadence::Weekly);

        assert!(matches!(target.cadence, TargetCadence::Weekly));
        assert!(target.updated_at > original_updated_at);
    }

    // ============================================
    // Display and Formatting Tests
    // ============================================

    #[test]
    fn test_cadence_display() {
        assert_eq!(TargetCadence::weekly().description(), "Weekly");
        assert_eq!(TargetCadence::monthly().description(), "Monthly");
        assert_eq!(TargetCadence::yearly().description(), "Yearly");
        assert_eq!(TargetCadence::custom(14).description(), "Every 14 days");
        assert_eq!(
            TargetCadence::by_date(NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()).description(),
            "By 2025-06-01"
        );
    }

    #[test]
    fn test_target_id_display() {
        let id = BudgetTargetId::new();
        let display = format!("{}", id);
        assert!(display.starts_with("tgt-"));
        assert_eq!(display.len(), 12); // "tgt-" + 8 hex chars
    }

    #[test]
    fn test_target_display() {
        let target = BudgetTarget::monthly(test_category_id(), Money::from_cents(50000));
        let display = format!("{}", target);
        assert!(display.contains("Monthly"));
    }

    // ============================================
    // Serialization Tests for All Cadence Types
    // ============================================

    #[test]
    fn test_weekly_cadence_serialization() {
        let target = BudgetTarget::weekly(test_category_id(), Money::from_cents(7000));
        let json = serde_json::to_string(&target).unwrap();
        let deserialized: BudgetTarget = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized.cadence, TargetCadence::Weekly));
    }

    #[test]
    fn test_yearly_cadence_serialization() {
        let target = BudgetTarget::yearly(test_category_id(), Money::from_cents(120000));
        let json = serde_json::to_string(&target).unwrap();
        let deserialized: BudgetTarget = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized.cadence, TargetCadence::Yearly));
    }

    #[test]
    fn test_custom_cadence_serialization() {
        let target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(10000),
            TargetCadence::custom(14),
        );
        let json = serde_json::to_string(&target).unwrap();
        let deserialized: BudgetTarget = serde_json::from_str(&json).unwrap();

        match deserialized.cadence {
            TargetCadence::Custom { days } => assert_eq!(days, 14),
            _ => panic!("Expected Custom cadence"),
        }
    }

    #[test]
    fn test_by_date_cadence_serialization() {
        let target_date = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
        let target = BudgetTarget::new(
            test_category_id(),
            Money::from_cents(100000),
            TargetCadence::by_date(target_date),
        );
        let json = serde_json::to_string(&target).unwrap();
        let deserialized: BudgetTarget = serde_json::from_str(&json).unwrap();

        match deserialized.cadence {
            TargetCadence::ByDate {
                target_date: date, ..
            } => assert_eq!(date, target_date),
            _ => panic!("Expected ByDate cadence"),
        }
    }
}
