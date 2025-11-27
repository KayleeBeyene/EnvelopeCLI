//! Account model
//!
//! Represents financial accounts (checking, savings, credit cards, etc.)

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::ids::AccountId;
use super::money::Money;

/// Type of financial account
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    /// Checking account
    Checking,
    /// Savings account
    Savings,
    /// Credit card
    Credit,
    /// Cash/wallet
    Cash,
    /// Investment account
    Investment,
    /// Line of credit
    LineOfCredit,
    /// Other account type
    Other,
}

impl AccountType {
    /// Returns true if this account type typically has a negative balance as normal
    /// (e.g., credit cards show debt as positive spending)
    pub fn is_liability(&self) -> bool {
        matches!(self, Self::Credit | Self::LineOfCredit)
    }

    /// Parse account type from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "checking" => Some(Self::Checking),
            "savings" => Some(Self::Savings),
            "credit" | "credit_card" | "creditcard" => Some(Self::Credit),
            "cash" => Some(Self::Cash),
            "investment" => Some(Self::Investment),
            "line_of_credit" | "lineofcredit" | "loc" => Some(Self::LineOfCredit),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

impl Default for AccountType {
    fn default() -> Self {
        Self::Checking
    }
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Checking => write!(f, "Checking"),
            Self::Savings => write!(f, "Savings"),
            Self::Credit => write!(f, "Credit Card"),
            Self::Cash => write!(f, "Cash"),
            Self::Investment => write!(f, "Investment"),
            Self::LineOfCredit => write!(f, "Line of Credit"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// A financial account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique identifier
    pub id: AccountId,

    /// Account name (e.g., "Chase Checking")
    pub name: String,

    /// Type of account
    #[serde(rename = "type")]
    pub account_type: AccountType,

    /// Whether this account is included in the budget
    /// Off-budget accounts (like investments) don't affect Available to Budget
    pub on_budget: bool,

    /// Whether this account is archived (soft-deleted)
    pub archived: bool,

    /// Opening balance when the account was created
    pub starting_balance: Money,

    /// Notes about this account
    #[serde(default)]
    pub notes: String,

    /// Date of last reconciliation
    pub last_reconciled_date: Option<NaiveDate>,

    /// Balance at last reconciliation
    pub last_reconciled_balance: Option<Money>,

    /// When the account was created
    pub created_at: DateTime<Utc>,

    /// When the account was last modified
    pub updated_at: DateTime<Utc>,

    /// Sort order for display
    #[serde(default)]
    pub sort_order: i32,
}

impl Account {
    /// Create a new account with default values
    pub fn new(name: impl Into<String>, account_type: AccountType) -> Self {
        let now = Utc::now();
        Self {
            id: AccountId::new(),
            name: name.into(),
            account_type,
            on_budget: true,
            archived: false,
            starting_balance: Money::zero(),
            notes: String::new(),
            last_reconciled_date: None,
            last_reconciled_balance: None,
            created_at: now,
            updated_at: now,
            sort_order: 0,
        }
    }

    /// Create a new account with a starting balance
    pub fn with_starting_balance(
        name: impl Into<String>,
        account_type: AccountType,
        starting_balance: Money,
    ) -> Self {
        let mut account = Self::new(name, account_type);
        account.starting_balance = starting_balance;
        account
    }

    /// Mark this account as archived
    pub fn archive(&mut self) {
        self.archived = true;
        self.updated_at = Utc::now();
    }

    /// Unarchive this account
    pub fn unarchive(&mut self) {
        self.archived = false;
        self.updated_at = Utc::now();
    }

    /// Set whether this account is on-budget
    pub fn set_on_budget(&mut self, on_budget: bool) {
        self.on_budget = on_budget;
        self.updated_at = Utc::now();
    }

    /// Record a reconciliation
    pub fn reconcile(&mut self, date: NaiveDate, balance: Money) {
        self.last_reconciled_date = Some(date);
        self.last_reconciled_balance = Some(balance);
        self.updated_at = Utc::now();
    }

    /// Validate the account
    pub fn validate(&self) -> Result<(), AccountValidationError> {
        if self.name.trim().is_empty() {
            return Err(AccountValidationError::EmptyName);
        }

        if self.name.len() > 100 {
            return Err(AccountValidationError::NameTooLong(self.name.len()));
        }

        Ok(())
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.account_type)
    }
}

/// Validation errors for accounts
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountValidationError {
    EmptyName,
    NameTooLong(usize),
}

impl fmt::Display for AccountValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(f, "Account name cannot be empty"),
            Self::NameTooLong(len) => {
                write!(f, "Account name too long ({} chars, max 100)", len)
            }
        }
    }
}

impl std::error::Error for AccountValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_account() {
        let account = Account::new("Checking", AccountType::Checking);
        assert_eq!(account.name, "Checking");
        assert_eq!(account.account_type, AccountType::Checking);
        assert!(account.on_budget);
        assert!(!account.archived);
        assert_eq!(account.starting_balance, Money::zero());
    }

    #[test]
    fn test_with_starting_balance() {
        let account = Account::with_starting_balance(
            "Savings",
            AccountType::Savings,
            Money::from_cents(100000),
        );
        assert_eq!(account.starting_balance.cents(), 100000);
    }

    #[test]
    fn test_archive() {
        let mut account = Account::new("Test", AccountType::Checking);
        assert!(!account.archived);

        account.archive();
        assert!(account.archived);

        account.unarchive();
        assert!(!account.archived);
    }

    #[test]
    fn test_validation() {
        let mut account = Account::new("Valid Name", AccountType::Checking);
        assert!(account.validate().is_ok());

        account.name = String::new();
        assert_eq!(
            account.validate(),
            Err(AccountValidationError::EmptyName)
        );

        account.name = "a".repeat(101);
        assert!(matches!(
            account.validate(),
            Err(AccountValidationError::NameTooLong(_))
        ));
    }

    #[test]
    fn test_account_type_parsing() {
        assert_eq!(AccountType::parse("checking"), Some(AccountType::Checking));
        assert_eq!(AccountType::parse("SAVINGS"), Some(AccountType::Savings));
        assert_eq!(AccountType::parse("credit_card"), Some(AccountType::Credit));
        assert_eq!(AccountType::parse("invalid"), None);
    }

    #[test]
    fn test_is_liability() {
        assert!(AccountType::Credit.is_liability());
        assert!(AccountType::LineOfCredit.is_liability());
        assert!(!AccountType::Checking.is_liability());
        assert!(!AccountType::Savings.is_liability());
    }

    #[test]
    fn test_serialization() {
        let account = Account::new("Test", AccountType::Checking);
        let json = serde_json::to_string(&account).unwrap();
        let deserialized: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(account.id, deserialized.id);
        assert_eq!(account.name, deserialized.name);
    }

    #[test]
    fn test_display() {
        let account = Account::new("My Checking", AccountType::Checking);
        assert_eq!(format!("{}", account), "My Checking (Checking)");
    }
}
