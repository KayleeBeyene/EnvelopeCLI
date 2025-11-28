//! Budget period representation
//!
//! Supports multiple period types: monthly, weekly, bi-weekly, and custom date ranges.

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a budget period
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum BudgetPeriod {
    /// Monthly period (e.g., "2025-01")
    Monthly { year: i32, month: u32 },

    /// ISO week period (e.g., "2025-W03")
    Weekly { year: i32, week: u32 },

    /// Bi-weekly period (identified by start date)
    BiWeekly { start_date: NaiveDate },

    /// Custom date range
    Custom { start: NaiveDate, end: NaiveDate },
}

impl BudgetPeriod {
    /// Create a monthly period
    pub fn monthly(year: i32, month: u32) -> Self {
        Self::Monthly { year, month }
    }

    /// Create a weekly period (ISO week)
    pub fn weekly(year: i32, week: u32) -> Self {
        Self::Weekly { year, week }
    }

    /// Create a bi-weekly period starting on the given date
    pub fn bi_weekly(start_date: NaiveDate) -> Self {
        Self::BiWeekly { start_date }
    }

    /// Create a custom period
    pub fn custom(start: NaiveDate, end: NaiveDate) -> Self {
        Self::Custom { start, end }
    }

    /// Get the current monthly period
    pub fn current_month() -> Self {
        let today = chrono::Local::now().date_naive();
        Self::Monthly {
            year: today.year(),
            month: today.month(),
        }
    }

    /// Get the current weekly period
    pub fn current_week() -> Self {
        let today = chrono::Local::now().date_naive();
        Self::Weekly {
            year: today.iso_week().year(),
            week: today.iso_week().week(),
        }
    }

    /// Get the start date of this period
    pub fn start_date(&self) -> NaiveDate {
        match self {
            Self::Monthly { year, month } => NaiveDate::from_ymd_opt(*year, *month, 1)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(*year, 1, 1).unwrap()),
            Self::Weekly { year, week } => NaiveDate::from_isoywd_opt(*year, *week, Weekday::Mon)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(*year, 1, 1).unwrap()),
            Self::BiWeekly { start_date } => *start_date,
            Self::Custom { start, .. } => *start,
        }
    }

    /// Get the end date of this period (inclusive)
    pub fn end_date(&self) -> NaiveDate {
        match self {
            Self::Monthly { year, month } => {
                let next_month = if *month == 12 {
                    NaiveDate::from_ymd_opt(*year + 1, 1, 1)
                } else {
                    NaiveDate::from_ymd_opt(*year, *month + 1, 1)
                };
                next_month.unwrap() - Duration::days(1)
            }
            Self::Weekly { year, week } => NaiveDate::from_isoywd_opt(*year, *week, Weekday::Sun)
                .unwrap_or_else(|| self.start_date() + Duration::days(6)),
            Self::BiWeekly { start_date } => *start_date + Duration::days(13),
            Self::Custom { end, .. } => *end,
        }
    }

    /// Check if a date falls within this period
    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start_date() && date <= self.end_date()
    }

    /// Get the next period
    pub fn next(&self) -> Self {
        match self {
            Self::Monthly { year, month } => {
                if *month == 12 {
                    Self::Monthly {
                        year: *year + 1,
                        month: 1,
                    }
                } else {
                    Self::Monthly {
                        year: *year,
                        month: *month + 1,
                    }
                }
            }
            Self::Weekly { year, week } => {
                // ISO weeks go from 1-52 or 1-53
                let max_week = NaiveDate::from_ymd_opt(*year, 12, 28)
                    .unwrap()
                    .iso_week()
                    .week();
                if *week >= max_week {
                    Self::Weekly {
                        year: *year + 1,
                        week: 1,
                    }
                } else {
                    Self::Weekly {
                        year: *year,
                        week: *week + 1,
                    }
                }
            }
            Self::BiWeekly { start_date } => Self::BiWeekly {
                start_date: *start_date + Duration::days(14),
            },
            Self::Custom { start, end } => {
                let duration = *end - *start;
                Self::Custom {
                    start: *end + Duration::days(1),
                    end: *end + duration + Duration::days(1),
                }
            }
        }
    }

    /// Get the previous period
    pub fn prev(&self) -> Self {
        match self {
            Self::Monthly { year, month } => {
                if *month == 1 {
                    Self::Monthly {
                        year: *year - 1,
                        month: 12,
                    }
                } else {
                    Self::Monthly {
                        year: *year,
                        month: *month - 1,
                    }
                }
            }
            Self::Weekly { year, week } => {
                if *week == 1 {
                    let prev_year = *year - 1;
                    let max_week = NaiveDate::from_ymd_opt(prev_year, 12, 28)
                        .unwrap()
                        .iso_week()
                        .week();
                    Self::Weekly {
                        year: prev_year,
                        week: max_week,
                    }
                } else {
                    Self::Weekly {
                        year: *year,
                        week: *week - 1,
                    }
                }
            }
            Self::BiWeekly { start_date } => Self::BiWeekly {
                start_date: *start_date - Duration::days(14),
            },
            Self::Custom { start, end } => {
                let duration = *end - *start;
                Self::Custom {
                    start: *start - duration - Duration::days(1),
                    end: *start - Duration::days(1),
                }
            }
        }
    }

    /// Parse a period string
    ///
    /// Formats:
    /// - Monthly: "2025-01"
    /// - Weekly: "2025-W03"
    /// - Custom: "2025-01-01..2025-01-15"
    pub fn parse(s: &str) -> Result<Self, PeriodParseError> {
        let s = s.trim();

        // Try weekly format first (contains W)
        if s.contains('W') {
            let parts: Vec<&str> = s.split("-W").collect();
            if parts.len() == 2 {
                let year: i32 = parts[0]
                    .parse()
                    .map_err(|_| PeriodParseError::InvalidFormat(s.to_string()))?;
                let week: u32 = parts[1]
                    .parse()
                    .map_err(|_| PeriodParseError::InvalidFormat(s.to_string()))?;
                return Ok(Self::Weekly { year, week });
            }
        }

        // Try custom range format (contains ..)
        if s.contains("..") {
            let parts: Vec<&str> = s.split("..").collect();
            if parts.len() == 2 {
                let start = NaiveDate::parse_from_str(parts[0], "%Y-%m-%d")
                    .map_err(|_| PeriodParseError::InvalidFormat(s.to_string()))?;
                let end = NaiveDate::parse_from_str(parts[1], "%Y-%m-%d")
                    .map_err(|_| PeriodParseError::InvalidFormat(s.to_string()))?;
                return Ok(Self::Custom { start, end });
            }
        }

        // Try monthly format (YYYY-MM)
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() == 2 {
            let year: i32 = parts[0]
                .parse()
                .map_err(|_| PeriodParseError::InvalidFormat(s.to_string()))?;
            let month: u32 = parts[1]
                .parse()
                .map_err(|_| PeriodParseError::InvalidFormat(s.to_string()))?;

            if !(1..=12).contains(&month) {
                return Err(PeriodParseError::InvalidMonth(month));
            }

            return Ok(Self::Monthly { year, month });
        }

        Err(PeriodParseError::InvalidFormat(s.to_string()))
    }
}

impl fmt::Display for BudgetPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Monthly { year, month } => write!(f, "{:04}-{:02}", year, month),
            Self::Weekly { year, week } => write!(f, "{:04}-W{:02}", year, week),
            Self::BiWeekly { start_date } => {
                let end = *start_date + Duration::days(13);
                write!(
                    f,
                    "{} - {}",
                    start_date.format("%Y-%m-%d"),
                    end.format("%Y-%m-%d")
                )
            }
            Self::Custom { start, end } => {
                write!(
                    f,
                    "{}..{}",
                    start.format("%Y-%m-%d"),
                    end.format("%Y-%m-%d")
                )
            }
        }
    }
}

impl Ord for BudgetPeriod {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start_date().cmp(&other.start_date())
    }
}

impl PartialOrd for BudgetPeriod {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Error type for period parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeriodParseError {
    InvalidFormat(String),
    InvalidMonth(u32),
}

impl fmt::Display for PeriodParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeriodParseError::InvalidFormat(s) => write!(f, "Invalid period format: {}", s),
            PeriodParseError::InvalidMonth(m) => write!(f, "Invalid month: {}", m),
        }
    }
}

impl std::error::Error for PeriodParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monthly_period() {
        let period = BudgetPeriod::monthly(2025, 1);
        assert_eq!(
            period.start_date(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
        );
        assert_eq!(
            period.end_date(),
            NaiveDate::from_ymd_opt(2025, 1, 31).unwrap()
        );
    }

    #[test]
    fn test_monthly_navigation() {
        let jan = BudgetPeriod::monthly(2025, 1);
        let feb = jan.next();
        assert_eq!(feb, BudgetPeriod::monthly(2025, 2));

        let dec = BudgetPeriod::monthly(2024, 12);
        let jan2025 = dec.next();
        assert_eq!(jan2025, BudgetPeriod::monthly(2025, 1));
    }

    #[test]
    fn test_weekly_period() {
        let period = BudgetPeriod::weekly(2025, 1);
        // ISO week 1 of 2025 starts on Monday December 30, 2024
        assert!(period.start_date() <= NaiveDate::from_ymd_opt(2025, 1, 5).unwrap());
    }

    #[test]
    fn test_contains() {
        let jan = BudgetPeriod::monthly(2025, 1);
        assert!(jan.contains(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()));
        assert!(!jan.contains(NaiveDate::from_ymd_opt(2025, 2, 1).unwrap()));
    }

    #[test]
    fn test_parse_monthly() {
        let period = BudgetPeriod::parse("2025-01").unwrap();
        assert_eq!(period, BudgetPeriod::monthly(2025, 1));
    }

    #[test]
    fn test_parse_weekly() {
        let period = BudgetPeriod::parse("2025-W03").unwrap();
        assert_eq!(period, BudgetPeriod::weekly(2025, 3));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", BudgetPeriod::monthly(2025, 1)), "2025-01");
        assert_eq!(format!("{}", BudgetPeriod::weekly(2025, 3)), "2025-W03");
    }

    #[test]
    fn test_serialization() {
        let period = BudgetPeriod::monthly(2025, 1);
        let json = serde_json::to_string(&period).unwrap();
        let deserialized: BudgetPeriod = serde_json::from_str(&json).unwrap();
        assert_eq!(period, deserialized);
    }
}
