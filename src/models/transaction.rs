//! Transaction model
//!
//! Represents financial transactions with support for splits, transfers,
//! and various statuses (pending, cleared, reconciled).

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::ids::{AccountId, CategoryId, PayeeId, TransactionId};
use super::money::Money;

/// Status of a transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    /// Transaction has not yet cleared the bank
    #[default]
    Pending,
    /// Transaction has cleared the bank
    Cleared,
    /// Transaction has been reconciled and is locked
    Reconciled,
}

impl TransactionStatus {
    /// Check if this transaction is locked (cannot be edited without unlocking)
    pub fn is_locked(&self) -> bool {
        matches!(self, Self::Reconciled)
    }
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Cleared => write!(f, "Cleared"),
            Self::Reconciled => write!(f, "Reconciled"),
        }
    }
}

/// A split portion of a transaction assigned to a specific category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Split {
    /// The category for this split portion
    pub category_id: CategoryId,

    /// The amount for this split (same sign as parent transaction)
    pub amount: Money,

    /// Optional memo for this split
    #[serde(default)]
    pub memo: String,
}

impl Split {
    /// Create a new split
    pub fn new(category_id: CategoryId, amount: Money) -> Self {
        Self {
            category_id,
            amount,
            memo: String::new(),
        }
    }

    /// Create a new split with a memo
    pub fn with_memo(category_id: CategoryId, amount: Money, memo: impl Into<String>) -> Self {
        Self {
            category_id,
            amount,
            memo: memo.into(),
        }
    }
}

/// A financial transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique identifier
    pub id: TransactionId,

    /// The account this transaction belongs to
    pub account_id: AccountId,

    /// Transaction date
    pub date: NaiveDate,

    /// Amount (positive for inflow, negative for outflow)
    pub amount: Money,

    /// Payee ID (optional)
    pub payee_id: Option<PayeeId>,

    /// Payee name (stored for display, even if payee_id is set)
    #[serde(default)]
    pub payee_name: String,

    /// Category ID (None if this is a split transaction or transfer)
    pub category_id: Option<CategoryId>,

    /// Split transactions - if non-empty, category_id should be None
    #[serde(default)]
    pub splits: Vec<Split>,

    /// Memo/notes
    #[serde(default)]
    pub memo: String,

    /// Transaction status
    #[serde(default)]
    pub status: TransactionStatus,

    /// If this is a transfer, the ID of the linked transaction in the other account
    pub transfer_transaction_id: Option<TransactionId>,

    /// Import ID for duplicate detection during CSV import
    pub import_id: Option<String>,

    /// When the transaction was created
    pub created_at: DateTime<Utc>,

    /// When the transaction was last modified
    pub updated_at: DateTime<Utc>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(account_id: AccountId, date: NaiveDate, amount: Money) -> Self {
        let now = Utc::now();
        Self {
            id: TransactionId::new(),
            account_id,
            date,
            amount,
            payee_id: None,
            payee_name: String::new(),
            category_id: None,
            splits: Vec::new(),
            memo: String::new(),
            status: TransactionStatus::Pending,
            transfer_transaction_id: None,
            import_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a transaction with all common fields
    pub fn with_details(
        account_id: AccountId,
        date: NaiveDate,
        amount: Money,
        payee_name: impl Into<String>,
        category_id: Option<CategoryId>,
        memo: impl Into<String>,
    ) -> Self {
        let mut txn = Self::new(account_id, date, amount);
        txn.payee_name = payee_name.into();
        txn.category_id = category_id;
        txn.memo = memo.into();
        txn
    }

    /// Check if this is a split transaction
    pub fn is_split(&self) -> bool {
        !self.splits.is_empty()
    }

    /// Check if this is a transfer
    pub fn is_transfer(&self) -> bool {
        self.transfer_transaction_id.is_some()
    }

    /// Check if this is an inflow (positive amount)
    pub fn is_inflow(&self) -> bool {
        self.amount.is_positive()
    }

    /// Check if this is an outflow (negative amount)
    pub fn is_outflow(&self) -> bool {
        self.amount.is_negative()
    }

    /// Check if this transaction is locked
    pub fn is_locked(&self) -> bool {
        self.status.is_locked()
    }

    /// Set the status
    pub fn set_status(&mut self, status: TransactionStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Clear the transaction (mark as cleared)
    pub fn clear(&mut self) {
        self.set_status(TransactionStatus::Cleared);
    }

    /// Mark as reconciled
    pub fn reconcile(&mut self) {
        self.set_status(TransactionStatus::Reconciled);
    }

    /// Add a split
    pub fn add_split(&mut self, split: Split) {
        self.splits.push(split);
        // When splits are added, category_id should be cleared
        self.category_id = None;
        self.updated_at = Utc::now();
    }

    /// Clear all splits and set a single category
    pub fn set_category(&mut self, category_id: CategoryId) {
        self.splits.clear();
        self.category_id = Some(category_id);
        self.updated_at = Utc::now();
    }

    /// Get the total of all splits (should equal transaction amount)
    pub fn splits_total(&self) -> Money {
        self.splits.iter().map(|s| s.amount).sum()
    }

    /// Validate the transaction
    pub fn validate(&self) -> Result<(), TransactionValidationError> {
        // If split, splits total must equal transaction amount
        if self.is_split() {
            let splits_total = self.splits_total();
            if splits_total != self.amount {
                return Err(TransactionValidationError::SplitsMismatch {
                    transaction_amount: self.amount,
                    splits_total,
                });
            }
        }

        // Can't have both category_id and splits
        if self.category_id.is_some() && !self.splits.is_empty() {
            return Err(TransactionValidationError::CategoryAndSplits);
        }

        // Transfers shouldn't have categories
        if self.is_transfer() && (self.category_id.is_some() || !self.splits.is_empty()) {
            return Err(TransactionValidationError::TransferWithCategory);
        }

        Ok(())
    }

    /// Generate an import ID for duplicate detection
    pub fn generate_import_id(&self) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.date.hash(&mut hasher);
        self.amount.cents().hash(&mut hasher);
        self.payee_name.hash(&mut hasher);
        format!("imp-{:016x}", hasher.finish())
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.date.format("%Y-%m-%d"),
            self.payee_name,
            self.amount
        )
    }
}

/// Validation errors for transactions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionValidationError {
    SplitsMismatch {
        transaction_amount: Money,
        splits_total: Money,
    },
    CategoryAndSplits,
    TransferWithCategory,
}

impl fmt::Display for TransactionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SplitsMismatch {
                transaction_amount,
                splits_total,
            } => write!(
                f,
                "Split totals ({}) do not match transaction amount ({})",
                splits_total, transaction_amount
            ),
            Self::CategoryAndSplits => {
                write!(f, "Transaction cannot have both a category and splits")
            }
            Self::TransferWithCategory => {
                write!(f, "Transfer transactions should not have a category")
            }
        }
    }
}

impl std::error::Error for TransactionValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_account_id() -> AccountId {
        AccountId::new()
    }

    fn test_category_id() -> CategoryId {
        CategoryId::new()
    }

    #[test]
    fn test_new_transaction() {
        let account_id = test_account_id();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let amount = Money::from_cents(-5000);

        let txn = Transaction::new(account_id, date, amount);
        assert_eq!(txn.account_id, account_id);
        assert_eq!(txn.date, date);
        assert_eq!(txn.amount, amount);
        assert_eq!(txn.status, TransactionStatus::Pending);
    }

    #[test]
    fn test_inflow_outflow() {
        let account_id = test_account_id();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let inflow = Transaction::new(account_id, date, Money::from_cents(1000));
        assert!(inflow.is_inflow());
        assert!(!inflow.is_outflow());

        let outflow = Transaction::new(account_id, date, Money::from_cents(-1000));
        assert!(!outflow.is_inflow());
        assert!(outflow.is_outflow());
    }

    #[test]
    fn test_status_transitions() {
        let account_id = test_account_id();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut txn = Transaction::new(account_id, date, Money::from_cents(-1000));

        assert!(!txn.is_locked());

        txn.clear();
        assert_eq!(txn.status, TransactionStatus::Cleared);
        assert!(!txn.is_locked());

        txn.reconcile();
        assert_eq!(txn.status, TransactionStatus::Reconciled);
        assert!(txn.is_locked());
    }

    #[test]
    fn test_split_transaction() {
        let account_id = test_account_id();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut txn = Transaction::new(account_id, date, Money::from_cents(-10000));

        let cat1 = test_category_id();
        let cat2 = test_category_id();

        txn.add_split(Split::new(cat1, Money::from_cents(-6000)));
        txn.add_split(Split::new(cat2, Money::from_cents(-4000)));

        assert!(txn.is_split());
        assert_eq!(txn.splits_total(), Money::from_cents(-10000));
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_split_validation_mismatch() {
        let account_id = test_account_id();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut txn = Transaction::new(account_id, date, Money::from_cents(-10000));

        let cat1 = test_category_id();
        txn.add_split(Split::new(cat1, Money::from_cents(-5000)));

        assert!(matches!(
            txn.validate(),
            Err(TransactionValidationError::SplitsMismatch { .. })
        ));
    }

    #[test]
    fn test_category_and_splits_validation() {
        let account_id = test_account_id();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut txn = Transaction::new(account_id, date, Money::from_cents(-10000));

        let cat1 = test_category_id();
        let cat2 = test_category_id();

        txn.category_id = Some(cat1);
        txn.splits.push(Split::new(cat2, Money::from_cents(-10000)));

        assert_eq!(
            txn.validate(),
            Err(TransactionValidationError::CategoryAndSplits)
        );
    }

    #[test]
    fn test_import_id_generation() {
        let account_id = test_account_id();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut txn = Transaction::new(account_id, date, Money::from_cents(-5000));
        txn.payee_name = "Test Store".to_string();

        let import_id = txn.generate_import_id();
        assert!(import_id.starts_with("imp-"));

        // Same transaction should generate same import ID
        let import_id2 = txn.generate_import_id();
        assert_eq!(import_id, import_id2);
    }

    #[test]
    fn test_serialization() {
        let account_id = test_account_id();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let txn = Transaction::with_details(
            account_id,
            date,
            Money::from_cents(-5000),
            "Test Store",
            Some(test_category_id()),
            "Test memo",
        );

        let json = serde_json::to_string(&txn).unwrap();
        let deserialized: Transaction = serde_json::from_str(&json).unwrap();
        assert_eq!(txn.id, deserialized.id);
        assert_eq!(txn.amount, deserialized.amount);
        assert_eq!(txn.payee_name, deserialized.payee_name);
    }

    #[test]
    fn test_display() {
        let account_id = test_account_id();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut txn = Transaction::new(account_id, date, Money::from_cents(-5000));
        txn.payee_name = "Test Store".to_string();

        assert_eq!(format!("{}", txn), "2025-01-15 Test Store -$50.00");
    }
}
