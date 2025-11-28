//! Money type for representing currency amounts
//!
//! Internally stores amounts in cents (i64) to avoid floating-point precision
//! issues. Provides safe arithmetic operations and formatting.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

/// Represents a monetary amount stored as cents (hundredths of the currency unit)
///
/// Using i64 cents avoids floating-point precision issues and supports
/// amounts up to approximately $92 quadrillion (both positive and negative).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Money(i64);

impl Money {
    /// Create a Money amount from cents
    ///
    /// # Examples
    /// ```
    /// use envelope_cli::models::Money;
    /// let amount = Money::from_cents(1050); // $10.50
    /// ```
    pub const fn from_cents(cents: i64) -> Self {
        Self(cents)
    }

    /// Create a Money amount from dollars and cents
    ///
    /// # Examples
    /// ```
    /// use envelope_cli::models::Money;
    /// let amount = Money::from_dollars_cents(10, 50); // $10.50
    /// ```
    pub const fn from_dollars_cents(dollars: i64, cents: i64) -> Self {
        Self(dollars * 100 + cents)
    }

    /// Create a zero Money amount
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Get the amount in cents
    pub const fn cents(&self) -> i64 {
        self.0
    }

    /// Get the whole dollars portion (truncated toward zero)
    pub const fn dollars(&self) -> i64 {
        self.0 / 100
    }

    /// Get the cents portion (0-99)
    pub const fn cents_part(&self) -> i64 {
        (self.0 % 100).abs()
    }

    /// Check if the amount is zero
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Check if the amount is positive
    pub const fn is_positive(&self) -> bool {
        self.0 > 0
    }

    /// Check if the amount is negative
    pub const fn is_negative(&self) -> bool {
        self.0 < 0
    }

    /// Get the absolute value
    pub const fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    /// Parse a money amount from a string
    ///
    /// Accepts formats: "10.50", "-10.50", "$10.50", "10", "1050" (cents)
    pub fn parse(s: &str) -> Result<Self, MoneyParseError> {
        let s = s.trim();

        // Handle negative sign at start
        let (negative, s) = if let Some(stripped) = s.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, s)
        };

        // Remove currency symbol if present
        let s = s.strip_prefix('$').unwrap_or(s);

        // Parse based on format
        let cents = if s.contains('.') {
            // Decimal format: "10.50"
            let parts: Vec<&str> = s.split('.').collect();
            if parts.len() != 2 {
                return Err(MoneyParseError::InvalidFormat(s.to_string()));
            }

            let dollars: i64 = parts[0]
                .parse()
                .map_err(|_| MoneyParseError::InvalidFormat(s.to_string()))?;

            // Pad or truncate cents to 2 digits
            let cents_str = parts[1];
            let cents: i64 = match cents_str.len() {
                0 => 0,
                1 => {
                    cents_str
                        .parse::<i64>()
                        .map_err(|_| MoneyParseError::InvalidFormat(s.to_string()))?
                        * 10
                }
                _ => cents_str[..2]
                    .parse()
                    .map_err(|_| MoneyParseError::InvalidFormat(s.to_string()))?,
            };

            dollars * 100 + cents
        } else {
            // Integer format - assume dollars
            s.parse::<i64>()
                .map_err(|_| MoneyParseError::InvalidFormat(s.to_string()))?
                * 100
        };

        Ok(Self(if negative { -cents } else { cents }))
    }

    /// Format with a currency symbol
    pub fn format_with_symbol(&self, symbol: &str) -> String {
        if self.is_negative() {
            format!(
                "-{}{}.{:02}",
                symbol,
                self.dollars().abs(),
                self.cents_part()
            )
        } else {
            format!("{}{}.{:02}", symbol, self.dollars(), self.cents_part())
        }
    }
}

impl Default for Money {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_negative() {
            write!(f, "-${}.{:02}", self.dollars().abs(), self.cents_part())
        } else {
            write!(f, "${}.{:02}", self.dollars(), self.cents_part())
        }
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl Neg for Money {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl std::iter::Sum for Money {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Money::zero(), |acc, m| acc + m)
    }
}

/// Error type for money parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyParseError {
    InvalidFormat(String),
}

impl fmt::Display for MoneyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoneyParseError::InvalidFormat(s) => write!(f, "Invalid money format: {}", s),
        }
    }
}

impl std::error::Error for MoneyParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_cents() {
        let m = Money::from_cents(1050);
        assert_eq!(m.cents(), 1050);
        assert_eq!(m.dollars(), 10);
        assert_eq!(m.cents_part(), 50);
    }

    #[test]
    fn test_from_dollars_cents() {
        let m = Money::from_dollars_cents(10, 50);
        assert_eq!(m.cents(), 1050);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Money::from_cents(1050)), "$10.50");
        assert_eq!(format!("{}", Money::from_cents(0)), "$0.00");
        assert_eq!(format!("{}", Money::from_cents(-1050)), "-$10.50");
        assert_eq!(format!("{}", Money::from_cents(5)), "$0.05");
    }

    #[test]
    fn test_arithmetic() {
        let a = Money::from_cents(1000);
        let b = Money::from_cents(500);

        assert_eq!((a + b).cents(), 1500);
        assert_eq!((a - b).cents(), 500);
        assert_eq!((-a).cents(), -1000);
    }

    #[test]
    fn test_parse() {
        assert_eq!(Money::parse("10.50").unwrap().cents(), 1050);
        assert_eq!(Money::parse("$10.50").unwrap().cents(), 1050);
        assert_eq!(Money::parse("-10.50").unwrap().cents(), -1050);
        assert_eq!(Money::parse("10").unwrap().cents(), 1000);
        assert_eq!(Money::parse("10.5").unwrap().cents(), 1050);
        assert_eq!(Money::parse("0.05").unwrap().cents(), 5);
    }

    #[test]
    fn test_comparison() {
        let a = Money::from_cents(1000);
        let b = Money::from_cents(500);
        let c = Money::from_cents(1000);

        assert!(a > b);
        assert!(b < a);
        assert_eq!(a, c);
    }

    #[test]
    fn test_is_checks() {
        assert!(Money::zero().is_zero());
        assert!(Money::from_cents(100).is_positive());
        assert!(Money::from_cents(-100).is_negative());
    }

    #[test]
    fn test_sum() {
        let amounts = vec![
            Money::from_cents(100),
            Money::from_cents(200),
            Money::from_cents(300),
        ];
        let total: Money = amounts.into_iter().sum();
        assert_eq!(total.cents(), 600);
    }

    #[test]
    fn test_serialization() {
        let m = Money::from_cents(1050);
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(json, "1050");

        let deserialized: Money = serde_json::from_str(&json).unwrap();
        assert_eq!(m, deserialized);
    }
}
