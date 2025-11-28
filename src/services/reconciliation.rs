//! Reconciliation service
//!
//! Provides business logic for account reconciliation workflow including
//! starting reconciliation, calculating differences, completing reconciliation,
//! and creating adjustment transactions.

use chrono::NaiveDate;

use crate::audit::EntityType;
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{AccountId, CategoryId, Money, Transaction, TransactionId, TransactionStatus};
use crate::storage::Storage;

/// Service for reconciliation operations
pub struct ReconciliationService<'a> {
    storage: &'a Storage,
}

/// Represents an active reconciliation session
#[derive(Debug, Clone)]
pub struct ReconciliationSession {
    /// The account being reconciled
    pub account_id: AccountId,
    /// Statement date
    pub statement_date: NaiveDate,
    /// Statement ending balance
    pub statement_balance: Money,
    /// Current cleared balance (before reconciliation changes)
    pub starting_cleared_balance: Money,
}

/// Summary of current reconciliation state
#[derive(Debug, Clone)]
pub struct ReconciliationSummary {
    /// The reconciliation session
    pub session: ReconciliationSession,
    /// List of uncleared transactions
    pub uncleared_transactions: Vec<Transaction>,
    /// List of cleared (but not reconciled) transactions
    pub cleared_transactions: Vec<Transaction>,
    /// Current cleared balance (starting + cleared transactions)
    pub current_cleared_balance: Money,
    /// Difference between statement and cleared balance
    pub difference: Money,
    /// Whether reconciliation can be completed (difference is zero)
    pub can_complete: bool,
}

/// Result of completing reconciliation
#[derive(Debug)]
pub struct ReconciliationResult {
    /// Number of transactions marked as reconciled
    pub transactions_reconciled: usize,
    /// Whether an adjustment transaction was created
    pub adjustment_created: bool,
    /// The adjustment amount (if any)
    pub adjustment_amount: Option<Money>,
}

impl<'a> ReconciliationService<'a> {
    /// Create a new reconciliation service
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Start a reconciliation session for an account
    pub fn start(
        &self,
        account_id: AccountId,
        statement_date: NaiveDate,
        statement_balance: Money,
    ) -> EnvelopeResult<ReconciliationSession> {
        // Verify account exists
        let account = self
            .storage
            .accounts
            .get(account_id)?
            .ok_or_else(|| EnvelopeError::account_not_found(account_id.to_string()))?;

        if account.archived {
            return Err(EnvelopeError::Reconciliation(
                "Cannot reconcile an archived account".into(),
            ));
        }

        // Calculate current cleared balance (starting balance + reconciled transactions)
        let starting_cleared_balance = self.calculate_reconciled_balance(account_id)?;

        Ok(ReconciliationSession {
            account_id,
            statement_date,
            statement_balance,
            starting_cleared_balance,
        })
    }

    /// Get the current state of a reconciliation session
    pub fn get_summary(
        &self,
        session: &ReconciliationSession,
    ) -> EnvelopeResult<ReconciliationSummary> {
        let transactions = self
            .storage
            .transactions
            .get_by_account(session.account_id)?;

        let mut uncleared_transactions = Vec::new();
        let mut cleared_transactions = Vec::new();
        let mut cleared_total = Money::zero();

        for txn in transactions {
            match txn.status {
                TransactionStatus::Pending => {
                    uncleared_transactions.push(txn);
                }
                TransactionStatus::Cleared => {
                    cleared_total += txn.amount;
                    cleared_transactions.push(txn);
                }
                TransactionStatus::Reconciled => {
                    // Already reconciled, included in starting balance
                }
            }
        }

        // Sort by date
        uncleared_transactions.sort_by(|a, b| a.date.cmp(&b.date));
        cleared_transactions.sort_by(|a, b| a.date.cmp(&b.date));

        let current_cleared_balance = session.starting_cleared_balance + cleared_total;
        let difference = session.statement_balance - current_cleared_balance;
        let can_complete = difference.is_zero();

        Ok(ReconciliationSummary {
            session: session.clone(),
            uncleared_transactions,
            cleared_transactions,
            current_cleared_balance,
            difference,
            can_complete,
        })
    }

    /// Get uncleared transactions for an account (both pending and cleared but not reconciled)
    pub fn get_uncleared_transactions(
        &self,
        account_id: AccountId,
    ) -> EnvelopeResult<Vec<Transaction>> {
        let transactions = self.storage.transactions.get_by_account(account_id)?;
        let mut result: Vec<Transaction> = transactions
            .into_iter()
            .filter(|t| !matches!(t.status, TransactionStatus::Reconciled))
            .collect();
        result.sort_by(|a, b| a.date.cmp(&b.date));
        Ok(result)
    }

    /// Calculate the difference between statement balance and current cleared balance
    pub fn get_difference(&self, session: &ReconciliationSession) -> EnvelopeResult<Money> {
        let summary = self.get_summary(session)?;
        Ok(summary.difference)
    }

    /// Clear a transaction during reconciliation
    pub fn clear_transaction(&self, transaction_id: TransactionId) -> EnvelopeResult<Transaction> {
        let mut txn = self
            .storage
            .transactions
            .get(transaction_id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(transaction_id.to_string()))?;

        if txn.status == TransactionStatus::Reconciled {
            return Err(EnvelopeError::Reconciliation(
                "Transaction is already reconciled".into(),
            ));
        }

        let before = txn.clone();
        txn.set_status(TransactionStatus::Cleared);

        self.storage.transactions.upsert(txn.clone())?;
        self.storage.transactions.save()?;

        self.storage.log_update(
            EntityType::Transaction,
            txn.id.to_string(),
            Some(format!("{} {}", txn.date, txn.payee_name)),
            &before,
            &txn,
            Some(format!(
                "status: {} -> Cleared (reconciliation)",
                before.status
            )),
        )?;

        Ok(txn)
    }

    /// Unclear a transaction during reconciliation
    pub fn unclear_transaction(
        &self,
        transaction_id: TransactionId,
    ) -> EnvelopeResult<Transaction> {
        let mut txn = self
            .storage
            .transactions
            .get(transaction_id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(transaction_id.to_string()))?;

        if txn.status == TransactionStatus::Reconciled {
            return Err(EnvelopeError::Reconciliation(
                "Cannot unclear a reconciled transaction. Unlock it first.".into(),
            ));
        }

        let before = txn.clone();
        txn.set_status(TransactionStatus::Pending);

        self.storage.transactions.upsert(txn.clone())?;
        self.storage.transactions.save()?;

        self.storage.log_update(
            EntityType::Transaction,
            txn.id.to_string(),
            Some(format!("{} {}", txn.date, txn.payee_name)),
            &before,
            &txn,
            Some(format!(
                "status: {} -> Pending (reconciliation)",
                before.status
            )),
        )?;

        Ok(txn)
    }

    /// Complete reconciliation when difference is zero
    pub fn complete(
        &self,
        session: &ReconciliationSession,
    ) -> EnvelopeResult<ReconciliationResult> {
        let summary = self.get_summary(session)?;

        if !summary.can_complete {
            return Err(EnvelopeError::Reconciliation(format!(
                "Cannot complete reconciliation: difference is {} (must be zero)",
                summary.difference
            )));
        }

        self.complete_internal(session, &summary.cleared_transactions)
    }

    /// Complete reconciliation with a discrepancy by creating an adjustment transaction
    pub fn complete_with_adjustment(
        &self,
        session: &ReconciliationSession,
        adjustment_category_id: Option<CategoryId>,
    ) -> EnvelopeResult<ReconciliationResult> {
        let summary = self.get_summary(session)?;

        if summary.can_complete {
            // No adjustment needed
            return self.complete(session);
        }

        // Verify category exists if provided
        if let Some(cat_id) = adjustment_category_id {
            self.storage
                .categories
                .get_category(cat_id)?
                .ok_or_else(|| EnvelopeError::category_not_found(cat_id.to_string()))?;
        }

        // Create adjustment transaction
        let adjustment_amount = summary.difference;
        let adjustment = self.create_adjustment_transaction(
            session.account_id,
            session.statement_date,
            adjustment_amount,
            adjustment_category_id,
        )?;

        // Now complete with the adjustment included
        let mut transactions_to_reconcile = summary.cleared_transactions;
        transactions_to_reconcile.push(adjustment);

        let result = self.complete_internal(session, &transactions_to_reconcile)?;

        Ok(ReconciliationResult {
            transactions_reconciled: result.transactions_reconciled,
            adjustment_created: true,
            adjustment_amount: Some(adjustment_amount),
        })
    }

    /// Create an adjustment transaction for reconciliation discrepancies
    pub fn create_adjustment_transaction(
        &self,
        account_id: AccountId,
        date: NaiveDate,
        amount: Money,
        category_id: Option<CategoryId>,
    ) -> EnvelopeResult<Transaction> {
        let mut txn = Transaction::new(account_id, date, amount);
        txn.payee_name = "Reconciliation Adjustment".to_string();
        txn.memo = "Created during reconciliation to match statement balance".to_string();
        txn.category_id = category_id;
        txn.status = TransactionStatus::Cleared;

        // Validate
        txn.validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        // Save
        self.storage.transactions.upsert(txn.clone())?;
        self.storage.transactions.save()?;

        // Audit log
        self.storage.log_create(
            EntityType::Transaction,
            txn.id.to_string(),
            Some(format!(
                "Reconciliation adjustment {} for account {}",
                amount, account_id
            )),
            &txn,
        )?;

        Ok(txn)
    }

    /// Internal method to complete reconciliation
    fn complete_internal(
        &self,
        session: &ReconciliationSession,
        transactions_to_reconcile: &[Transaction],
    ) -> EnvelopeResult<ReconciliationResult> {
        let mut count = 0;

        for txn in transactions_to_reconcile {
            let mut updated_txn = txn.clone();
            let before = txn.clone();
            updated_txn.set_status(TransactionStatus::Reconciled);

            self.storage.transactions.upsert(updated_txn.clone())?;

            self.storage.log_update(
                EntityType::Transaction,
                updated_txn.id.to_string(),
                Some(format!("{} {}", updated_txn.date, updated_txn.payee_name)),
                &before,
                &updated_txn,
                Some("status: Cleared -> Reconciled (reconciliation complete)".to_string()),
            )?;

            count += 1;
        }

        self.storage.transactions.save()?;

        // Update account's reconciliation info
        let mut account = self
            .storage
            .accounts
            .get(session.account_id)?
            .ok_or_else(|| EnvelopeError::account_not_found(session.account_id.to_string()))?;

        let before_account = account.clone();
        account.reconcile(session.statement_date, session.statement_balance);

        self.storage.accounts.upsert(account.clone())?;
        self.storage.accounts.save()?;

        self.storage.log_update(
            EntityType::Account,
            account.id.to_string(),
            Some(account.name.clone()),
            &before_account,
            &account,
            Some(format!(
                "reconciled: date={}, balance={}",
                session.statement_date, session.statement_balance
            )),
        )?;

        Ok(ReconciliationResult {
            transactions_reconciled: count,
            adjustment_created: false,
            adjustment_amount: None,
        })
    }

    /// Calculate the reconciled balance for an account
    /// (starting balance + all reconciled transactions)
    fn calculate_reconciled_balance(&self, account_id: AccountId) -> EnvelopeResult<Money> {
        let account = self
            .storage
            .accounts
            .get(account_id)?
            .ok_or_else(|| EnvelopeError::account_not_found(account_id.to_string()))?;

        let transactions = self.storage.transactions.get_by_account(account_id)?;
        let reconciled_total: Money = transactions
            .iter()
            .filter(|t| t.status == TransactionStatus::Reconciled)
            .map(|t| t.amount)
            .sum();

        Ok(account.starting_balance + reconciled_total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::EnvelopePaths;
    use crate::models::{Account, AccountType};
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, Storage) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        let mut storage = Storage::new(paths).unwrap();
        storage.load_all().unwrap();
        (temp_dir, storage)
    }

    fn create_test_account(storage: &Storage) -> Account {
        let account = Account::with_starting_balance(
            "Test Checking",
            AccountType::Checking,
            Money::from_cents(100000), // $1000.00 starting balance
        );
        storage.accounts.upsert(account.clone()).unwrap();
        storage.accounts.save().unwrap();
        account
    }

    #[test]
    fn test_start_reconciliation() {
        let (_temp_dir, storage) = create_test_storage();
        let account = create_test_account(&storage);
        let service = ReconciliationService::new(&storage);

        let statement_date = NaiveDate::from_ymd_opt(2025, 1, 31).unwrap();
        let statement_balance = Money::from_cents(95000); // $950.00

        let session = service
            .start(account.id, statement_date, statement_balance)
            .unwrap();

        assert_eq!(session.account_id, account.id);
        assert_eq!(session.statement_date, statement_date);
        assert_eq!(session.statement_balance.cents(), 95000);
        assert_eq!(session.starting_cleared_balance.cents(), 100000);
    }

    #[test]
    fn test_reconciliation_summary() {
        let (_temp_dir, storage) = create_test_storage();
        let account = create_test_account(&storage);
        let service = ReconciliationService::new(&storage);

        // Add some transactions
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        // Pending transaction
        let txn1 = Transaction::new(account.id, date, Money::from_cents(-2000));
        storage.transactions.upsert(txn1).unwrap();

        // Cleared transaction
        let mut txn2 = Transaction::new(account.id, date, Money::from_cents(-5000));
        txn2.set_status(TransactionStatus::Cleared);
        storage.transactions.upsert(txn2).unwrap();

        storage.transactions.save().unwrap();

        let statement_date = NaiveDate::from_ymd_opt(2025, 1, 31).unwrap();
        let statement_balance = Money::from_cents(95000);

        let session = service
            .start(account.id, statement_date, statement_balance)
            .unwrap();
        let summary = service.get_summary(&session).unwrap();

        assert_eq!(summary.uncleared_transactions.len(), 1);
        assert_eq!(summary.cleared_transactions.len(), 1);
        // Current cleared = 100000 (starting) + (-5000) (cleared) = 95000
        assert_eq!(summary.current_cleared_balance.cents(), 95000);
        assert!(summary.difference.is_zero());
        assert!(summary.can_complete);
    }

    #[test]
    fn test_complete_reconciliation() {
        let (_temp_dir, storage) = create_test_storage();
        let account = create_test_account(&storage);
        let service = ReconciliationService::new(&storage);

        // Add a cleared transaction
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut txn = Transaction::new(account.id, date, Money::from_cents(-5000));
        txn.set_status(TransactionStatus::Cleared);
        storage.transactions.upsert(txn.clone()).unwrap();
        storage.transactions.save().unwrap();

        let statement_date = NaiveDate::from_ymd_opt(2025, 1, 31).unwrap();
        let statement_balance = Money::from_cents(95000);

        let session = service
            .start(account.id, statement_date, statement_balance)
            .unwrap();

        let result = service.complete(&session).unwrap();

        assert_eq!(result.transactions_reconciled, 1);
        assert!(!result.adjustment_created);

        // Verify transaction is now reconciled
        let updated_txn = storage.transactions.get(txn.id).unwrap().unwrap();
        assert_eq!(updated_txn.status, TransactionStatus::Reconciled);

        // Verify account reconciliation info updated
        let updated_account = storage.accounts.get(account.id).unwrap().unwrap();
        assert_eq!(updated_account.last_reconciled_date, Some(statement_date));
        assert_eq!(
            updated_account.last_reconciled_balance,
            Some(statement_balance)
        );
    }

    #[test]
    fn test_complete_with_adjustment() {
        let (_temp_dir, storage) = create_test_storage();
        let account = create_test_account(&storage);
        let service = ReconciliationService::new(&storage);

        // No cleared transactions, but statement shows different balance
        let statement_date = NaiveDate::from_ymd_opt(2025, 1, 31).unwrap();
        let statement_balance = Money::from_cents(99000); // $10.00 less than starting

        let session = service
            .start(account.id, statement_date, statement_balance)
            .unwrap();

        // Should have a discrepancy
        let summary = service.get_summary(&session).unwrap();
        assert!(!summary.can_complete);
        assert_eq!(summary.difference.cents(), -1000); // Need -$10.00 adjustment

        // Complete with adjustment
        let result = service.complete_with_adjustment(&session, None).unwrap();

        assert!(result.adjustment_created);
        assert_eq!(result.adjustment_amount.unwrap().cents(), -1000);
    }

    #[test]
    fn test_cannot_complete_without_zero_difference() {
        let (_temp_dir, storage) = create_test_storage();
        let account = create_test_account(&storage);
        let service = ReconciliationService::new(&storage);

        let statement_date = NaiveDate::from_ymd_opt(2025, 1, 31).unwrap();
        let statement_balance = Money::from_cents(99000); // Different from starting

        let session = service
            .start(account.id, statement_date, statement_balance)
            .unwrap();

        let result = service.complete(&session);
        assert!(matches!(result, Err(EnvelopeError::Reconciliation(_))));
    }

    #[test]
    fn test_clear_unclear_transaction() {
        let (_temp_dir, storage) = create_test_storage();
        let account = create_test_account(&storage);
        let service = ReconciliationService::new(&storage);

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let txn = Transaction::new(account.id, date, Money::from_cents(-5000));
        storage.transactions.upsert(txn.clone()).unwrap();
        storage.transactions.save().unwrap();

        // Clear the transaction
        let cleared = service.clear_transaction(txn.id).unwrap();
        assert_eq!(cleared.status, TransactionStatus::Cleared);

        // Unclear it
        let uncleared = service.unclear_transaction(txn.id).unwrap();
        assert_eq!(uncleared.status, TransactionStatus::Pending);
    }
}
