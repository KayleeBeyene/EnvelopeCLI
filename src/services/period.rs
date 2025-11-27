//! Budget period service
//!
//! Provides period management including navigation, validation, and
//! period-specific operations.

use crate::config::settings::{BudgetPeriodType, Settings};
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::BudgetPeriod;
use chrono::{Datelike, Duration, Local, NaiveDate};

/// Service for budget period management
pub struct PeriodService<'a> {
    settings: &'a Settings,
}

impl<'a> PeriodService<'a> {
    /// Create a new period service
    pub fn new(settings: &'a Settings) -> Self {
        Self { settings }
    }

    /// Get the current period based on user preferences
    pub fn current_period(&self) -> BudgetPeriod {
        let today = Local::now().date_naive();
        self.period_for_date(today)
    }

    /// Get the period containing a specific date
    pub fn period_for_date(&self, date: NaiveDate) -> BudgetPeriod {
        match self.settings.budget_period_type {
            BudgetPeriodType::Monthly => BudgetPeriod::monthly(date.year(), date.month()),
            BudgetPeriodType::Weekly => {
                BudgetPeriod::weekly(date.iso_week().year(), date.iso_week().week())
            }
            BudgetPeriodType::BiWeekly => {
                // For bi-weekly, we need to find the start date
                // Using first Monday of the year as anchor
                let anchor = self.get_biweekly_anchor(date.year());
                let days_since_anchor = (date - anchor).num_days();
                let periods_since_anchor = days_since_anchor / 14;
                let period_start = anchor + Duration::days(periods_since_anchor * 14);
                BudgetPeriod::bi_weekly(period_start)
            }
        }
    }

    /// Get the next period after the given one
    pub fn next_period(&self, period: &BudgetPeriod) -> BudgetPeriod {
        period.next()
    }

    /// Get the previous period before the given one
    pub fn previous_period(&self, period: &BudgetPeriod) -> BudgetPeriod {
        period.prev()
    }

    /// Get the anchor date for bi-weekly calculations (first Monday of the year)
    fn get_biweekly_anchor(&self, year: i32) -> NaiveDate {
        let jan_1 = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let days_until_monday = (7 - jan_1.weekday().num_days_from_monday()) % 7;
        jan_1 + Duration::days(days_until_monday as i64)
    }

    /// Parse a period string or get current period
    pub fn parse_or_current(&self, period_str: Option<&str>) -> EnvelopeResult<BudgetPeriod> {
        match period_str {
            Some(s) => self.parse(s),
            None => Ok(self.current_period()),
        }
    }

    /// Parse a period string according to user preferences
    ///
    /// Formats supported:
    /// - Monthly: "2025-01", "January 2025", "Jan", "last", "next"
    /// - Weekly: "2025-W03", "W3", "last", "next"
    /// - Date range: "2025-01-01..2025-01-14"
    pub fn parse(&self, s: &str) -> EnvelopeResult<BudgetPeriod> {
        let s_lower = s.trim().to_lowercase();

        // Handle relative references
        if s_lower == "current" || s_lower == "now" || s_lower == "this" {
            return Ok(self.current_period());
        }

        if s_lower == "last" || s_lower == "previous" || s_lower == "prev" {
            return Ok(self.previous_period(&self.current_period()));
        }

        if s_lower == "next" {
            return Ok(self.next_period(&self.current_period()));
        }

        // Handle month names
        if let Some(period) = self.parse_month_name(&s_lower) {
            return Ok(period);
        }

        // Handle standard period format (preserve original case for weekly format)
        BudgetPeriod::parse(s.trim())
            .map_err(|_| EnvelopeError::Validation(format!("Invalid period format: {}", s)))
    }

    /// Parse month names like "January", "Jan", etc.
    fn parse_month_name(&self, s: &str) -> Option<BudgetPeriod> {
        let months = [
            ("january", 1),
            ("jan", 1),
            ("february", 2),
            ("feb", 2),
            ("march", 3),
            ("mar", 3),
            ("april", 4),
            ("apr", 4),
            ("may", 5),
            ("june", 6),
            ("jun", 6),
            ("july", 7),
            ("jul", 7),
            ("august", 8),
            ("aug", 8),
            ("september", 9),
            ("sep", 9),
            ("sept", 9),
            ("october", 10),
            ("oct", 10),
            ("november", 11),
            ("nov", 11),
            ("december", 12),
            ("dec", 12),
        ];

        for (name, month) in months {
            if s.starts_with(name) {
                // Check if year is specified (e.g., "January 2025" or "Jan 2025")
                let rest = s[name.len()..].trim();
                let year = if rest.is_empty() {
                    // Use current year, or previous year if month is in the future
                    let today = Local::now().date_naive();
                    if month > today.month() {
                        today.year() - 1
                    } else {
                        today.year()
                    }
                } else {
                    rest.parse().ok()?
                };

                return Some(BudgetPeriod::monthly(year, month));
            }
        }

        None
    }

    /// Get a list of periods for display (e.g., last 6 months)
    pub fn recent_periods(&self, count: usize) -> Vec<BudgetPeriod> {
        let mut periods = Vec::with_capacity(count);
        let mut current = self.current_period();

        for _ in 0..count {
            periods.push(current.clone());
            current = self.previous_period(&current);
        }

        periods.reverse();
        periods
    }

    /// Get a list of upcoming periods (current + future)
    pub fn upcoming_periods(&self, count: usize) -> Vec<BudgetPeriod> {
        let mut periods = Vec::with_capacity(count);
        let mut current = self.current_period();

        for _ in 0..count {
            periods.push(current.clone());
            current = self.next_period(&current);
        }

        periods
    }

    /// Format a period for display
    pub fn format_period(&self, period: &BudgetPeriod) -> String {
        period.to_string()
    }

    /// Format a period in a human-friendly way
    pub fn format_period_friendly(&self, period: &BudgetPeriod) -> String {
        match period {
            BudgetPeriod::Monthly { year, month } => {
                let month_names = [
                    "January",
                    "February",
                    "March",
                    "April",
                    "May",
                    "June",
                    "July",
                    "August",
                    "September",
                    "October",
                    "November",
                    "December",
                ];
                let month_name = month_names[(*month - 1) as usize];
                format!("{} {}", month_name, year)
            }
            BudgetPeriod::Weekly { year, week } => {
                format!("Week {} of {}", week, year)
            }
            BudgetPeriod::BiWeekly { start_date } => {
                let end_date = *start_date + Duration::days(13);
                format!("{} - {}", start_date.format("%b %d"), end_date.format("%b %d, %Y"))
            }
            BudgetPeriod::Custom { start, end } => {
                format!("{} to {}", start.format("%Y-%m-%d"), end.format("%Y-%m-%d"))
            }
        }
    }

    /// Check if a period is the current period
    pub fn is_current(&self, period: &BudgetPeriod) -> bool {
        *period == self.current_period()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_settings() -> Settings {
        Settings::default()
    }

    #[test]
    fn test_current_period() {
        let settings = default_settings();
        let service = PeriodService::new(&settings);

        let period = service.current_period();
        let today = Local::now().date_naive();

        // Should be a monthly period containing today
        assert!(period.contains(today));
    }

    #[test]
    fn test_period_navigation() {
        let settings = default_settings();
        let service = PeriodService::new(&settings);

        let jan = BudgetPeriod::monthly(2025, 1);
        let feb = service.next_period(&jan);
        let dec = service.previous_period(&jan);

        assert_eq!(feb, BudgetPeriod::monthly(2025, 2));
        assert_eq!(dec, BudgetPeriod::monthly(2024, 12));
    }

    #[test]
    fn test_parse_relative() {
        let settings = default_settings();
        let service = PeriodService::new(&settings);

        let current = service.current_period();

        assert_eq!(service.parse("current").unwrap(), current);
        assert_eq!(service.parse("now").unwrap(), current);
        assert_eq!(service.parse("last").unwrap(), service.previous_period(&current));
        assert_eq!(service.parse("next").unwrap(), service.next_period(&current));
    }

    #[test]
    fn test_parse_standard() {
        let settings = default_settings();
        let service = PeriodService::new(&settings);

        assert_eq!(
            service.parse("2025-01").unwrap(),
            BudgetPeriod::monthly(2025, 1)
        );
        assert_eq!(
            service.parse("2025-W03").unwrap(),
            BudgetPeriod::weekly(2025, 3)
        );
    }

    #[test]
    fn test_parse_month_name() {
        let settings = default_settings();
        let service = PeriodService::new(&settings);

        let jan2025 = service.parse("January 2025").unwrap();
        assert_eq!(jan2025, BudgetPeriod::monthly(2025, 1));

        let mar2025 = service.parse("Mar 2025").unwrap();
        assert_eq!(mar2025, BudgetPeriod::monthly(2025, 3));
    }

    #[test]
    fn test_recent_periods() {
        let settings = default_settings();
        let service = PeriodService::new(&settings);

        let recent = service.recent_periods(3);
        assert_eq!(recent.len(), 3);

        // Should be in chronological order
        assert!(recent[0] < recent[1]);
        assert!(recent[1] < recent[2]);

        // Last one should be current
        assert!(service.is_current(&recent[2]));
    }

    #[test]
    fn test_format_period_friendly() {
        let settings = default_settings();
        let service = PeriodService::new(&settings);

        let jan = BudgetPeriod::monthly(2025, 1);
        assert_eq!(service.format_period_friendly(&jan), "January 2025");

        let week3 = BudgetPeriod::weekly(2025, 3);
        assert_eq!(service.format_period_friendly(&week3), "Week 3 of 2025");
    }
}
