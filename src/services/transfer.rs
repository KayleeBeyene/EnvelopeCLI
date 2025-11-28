//! Transfer service
//!
//! Provides business logic for transfers between accounts.
//! Transfers create linked transaction pairs - an outflow from the source
//! account and an inflow to the destination account.

use chrono::{NaiveDate, Utc};

use crate::audit::EntityType;
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{Account, AccountId, Money, Transaction, TransactionId};

// Note: Transaction model uses transfer_transaction_id to link paired transfer transactions.
// The target account can be determined by looking up the linked transaction.
use crate::storage::Storage;

/// Service for managing transfers between accounts
pub struct TransferService<'a> {
    storage: &'a Storage,
}

/// Result of creating a transfer
#[derive(Debug, Clone)]
pub struct TransferResult {
    /// The outflow transaction (from source account)
    pub from_transaction: Transaction,
    /// The inflow transaction (to destination account)
    pub to_transaction: Transaction,
}

impl<'a> TransferService<'a> {
    /// Create a new transfer service
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Create a transfer between two accounts
    ///
    /// This creates two linked transactions:
    /// - An outflow (negative amount) from the source account
    /// - An inflow (positive amount) to the destination account
    pub fn create_transfer(
        &self,
        from_account_id: AccountId,
        to_account_id: AccountId,
        amount: Money,
        date: NaiveDate,
        memo: Option<String>,
    ) -> EnvelopeResult<TransferResult> {
        // Validate amount is positive
        if amount.is_zero() {
            return Err(EnvelopeError::Validation(
                "Transfer amount must be non-zero".into(),
            ));
        }
        if amount.is_negative() {
            return Err(EnvelopeError::Validation(
                "Transfer amount must be positive".into(),
            ));
        }

        // Can't transfer to the same account
        if from_account_id == to_account_id {
            return Err(EnvelopeError::Validation(
                "Cannot transfer to the same account".into(),
            ));
        }

        // Verify both accounts exist and are not archived
        let from_account = self.get_active_account(from_account_id)?;
        let to_account = self.get_active_account(to_account_id)?;

        // Create the outflow transaction (from source)
        let mut from_txn = Transaction::new(from_account_id, date, -amount);
        from_txn.payee_name = format!("Transfer to {}", to_account.name);
        if let Some(m) = &memo {
            from_txn.memo.clone_from(m);
        }

        // Create the inflow transaction (to destination)
        let mut to_txn = Transaction::new(to_account_id, date, amount);
        to_txn.payee_name = format!("Transfer from {}", from_account.name);
        if let Some(ref m) = memo {
            to_txn.memo = m.clone();
        }

        // Link them together
        from_txn.transfer_transaction_id = Some(to_txn.id);
        to_txn.transfer_transaction_id = Some(from_txn.id);

        // Validate both transactions
        from_txn
            .validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;
        to_txn
            .validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        // Save both transactions
        self.storage.transactions.upsert(from_txn.clone())?;
        self.storage.transactions.upsert(to_txn.clone())?;
        self.storage.transactions.save()?;

        // Audit log for both
        self.storage.log_create(
            EntityType::Transaction,
            from_txn.id.to_string(),
            Some(format!("Transfer to {}", to_account.name)),
            &from_txn,
        )?;

        self.storage.log_create(
            EntityType::Transaction,
            to_txn.id.to_string(),
            Some(format!("Transfer from {}", from_account.name)),
            &to_txn,
        )?;

        Ok(TransferResult {
            from_transaction: from_txn,
            to_transaction: to_txn,
        })
    }

    /// Get the linked transaction for a transfer
    pub fn get_linked_transaction(
        &self,
        transaction_id: TransactionId,
    ) -> EnvelopeResult<Option<Transaction>> {
        let txn = self
            .storage
            .transactions
            .get(transaction_id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(transaction_id.to_string()))?;

        if let Some(linked_id) = txn.transfer_transaction_id {
            self.storage.transactions.get(linked_id)
        } else {
            Ok(None)
        }
    }

    /// Update a transfer's amount
    ///
    /// This updates both the source and destination transactions to maintain consistency.
    pub fn update_transfer_amount(
        &self,
        transaction_id: TransactionId,
        new_amount: Money,
    ) -> EnvelopeResult<TransferResult> {
        if new_amount.is_zero() {
            return Err(EnvelopeError::Validation(
                "Transfer amount must be non-zero".into(),
            ));
        }

        let mut txn = self
            .storage
            .transactions
            .get(transaction_id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(transaction_id.to_string()))?;

        if !txn.is_transfer() {
            return Err(EnvelopeError::Validation(
                "Transaction is not a transfer".into(),
            ));
        }

        if txn.is_locked() {
            return Err(EnvelopeError::Locked(format!(
                "Transaction {} is reconciled and cannot be edited",
                transaction_id
            )));
        }

        let linked_id = txn.transfer_transaction_id.ok_or_else(|| {
            EnvelopeError::Validation("Transfer has no linked transaction".into())
        })?;

        let mut linked_txn = self
            .storage
            .transactions
            .get(linked_id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(linked_id.to_string()))?;

        if linked_txn.is_locked() {
            return Err(EnvelopeError::Locked(format!(
                "Linked transaction {} is reconciled and cannot be edited",
                linked_id
            )));
        }

        let txn_before = txn.clone();
        let linked_before = linked_txn.clone();

        // Determine which transaction is the outflow (negative) and which is the inflow (positive)
        let amount = new_amount.abs();
        if txn.amount.is_negative() {
            // txn is the outflow
            txn.amount = -amount;
            linked_txn.amount = amount;
        } else {
            // txn is the inflow
            txn.amount = amount;
            linked_txn.amount = -amount;
        }

        txn.updated_at = Utc::now();
        linked_txn.updated_at = Utc::now();

        // Validate both
        txn.validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;
        linked_txn
            .validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        // Save both
        self.storage.transactions.upsert(txn.clone())?;
        self.storage.transactions.upsert(linked_txn.clone())?;
        self.storage.transactions.save()?;

        // Audit log both
        self.storage.log_update(
            EntityType::Transaction,
            txn.id.to_string(),
            Some(txn.payee_name.clone()),
            &txn_before,
            &txn,
            Some(format!(
                "transfer amount: {} -> {}",
                txn_before.amount, txn.amount
            )),
        )?;

        self.storage.log_update(
            EntityType::Transaction,
            linked_txn.id.to_string(),
            Some(linked_txn.payee_name.clone()),
            &linked_before,
            &linked_txn,
            Some(format!(
                "transfer amount: {} -> {}",
                linked_before.amount, linked_txn.amount
            )),
        )?;

        // Return in consistent order (outflow first)
        if txn.amount.is_negative() {
            Ok(TransferResult {
                from_transaction: txn,
                to_transaction: linked_txn,
            })
        } else {
            Ok(TransferResult {
                from_transaction: linked_txn,
                to_transaction: txn,
            })
        }
    }

    /// Update a transfer's date
    ///
    /// This updates both the source and destination transactions to maintain consistency.
    pub fn update_transfer_date(
        &self,
        transaction_id: TransactionId,
        new_date: NaiveDate,
    ) -> EnvelopeResult<TransferResult> {
        let mut txn = self
            .storage
            .transactions
            .get(transaction_id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(transaction_id.to_string()))?;

        if !txn.is_transfer() {
            return Err(EnvelopeError::Validation(
                "Transaction is not a transfer".into(),
            ));
        }

        if txn.is_locked() {
            return Err(EnvelopeError::Locked(format!(
                "Transaction {} is reconciled and cannot be edited",
                transaction_id
            )));
        }

        let linked_id = txn.transfer_transaction_id.ok_or_else(|| {
            EnvelopeError::Validation("Transfer has no linked transaction".into())
        })?;

        let mut linked_txn = self
            .storage
            .transactions
            .get(linked_id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(linked_id.to_string()))?;

        if linked_txn.is_locked() {
            return Err(EnvelopeError::Locked(format!(
                "Linked transaction {} is reconciled and cannot be edited",
                linked_id
            )));
        }

        let txn_before = txn.clone();
        let linked_before = linked_txn.clone();

        txn.date = new_date;
        linked_txn.date = new_date;
        txn.updated_at = Utc::now();
        linked_txn.updated_at = Utc::now();

        // Save both
        self.storage.transactions.upsert(txn.clone())?;
        self.storage.transactions.upsert(linked_txn.clone())?;
        self.storage.transactions.save()?;

        // Audit log both
        self.storage.log_update(
            EntityType::Transaction,
            txn.id.to_string(),
            Some(txn.payee_name.clone()),
            &txn_before,
            &txn,
            Some(format!("date: {} -> {}", txn_before.date, txn.date)),
        )?;

        self.storage.log_update(
            EntityType::Transaction,
            linked_txn.id.to_string(),
            Some(linked_txn.payee_name.clone()),
            &linked_before,
            &linked_txn,
            Some(format!(
                "date: {} -> {}",
                linked_before.date, linked_txn.date
            )),
        )?;

        // Return in consistent order (outflow first)
        if txn.amount.is_negative() {
            Ok(TransferResult {
                from_transaction: txn,
                to_transaction: linked_txn,
            })
        } else {
            Ok(TransferResult {
                from_transaction: linked_txn,
                to_transaction: txn,
            })
        }
    }

    /// Delete a transfer (both transactions)
    pub fn delete_transfer(&self, transaction_id: TransactionId) -> EnvelopeResult<TransferResult> {
        let txn = self
            .storage
            .transactions
            .get(transaction_id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(transaction_id.to_string()))?;

        if !txn.is_transfer() {
            return Err(EnvelopeError::Validation(
                "Transaction is not a transfer".into(),
            ));
        }

        if txn.is_locked() {
            return Err(EnvelopeError::Locked(format!(
                "Transaction {} is reconciled and cannot be deleted",
                transaction_id
            )));
        }

        let linked_id = txn.transfer_transaction_id.ok_or_else(|| {
            EnvelopeError::Validation("Transfer has no linked transaction".into())
        })?;

        let linked_txn = self
            .storage
            .transactions
            .get(linked_id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(linked_id.to_string()))?;

        if linked_txn.is_locked() {
            return Err(EnvelopeError::Locked(format!(
                "Linked transaction {} is reconciled and cannot be deleted",
                linked_id
            )));
        }

        // Delete both
        self.storage.transactions.delete(txn.id)?;
        self.storage.transactions.delete(linked_txn.id)?;
        self.storage.transactions.save()?;

        // Audit log both
        self.storage.log_delete(
            EntityType::Transaction,
            txn.id.to_string(),
            Some(txn.payee_name.clone()),
            &txn,
        )?;

        self.storage.log_delete(
            EntityType::Transaction,
            linked_txn.id.to_string(),
            Some(linked_txn.payee_name.clone()),
            &linked_txn,
        )?;

        // Return in consistent order (outflow first)
        if txn.amount.is_negative() {
            Ok(TransferResult {
                from_transaction: txn,
                to_transaction: linked_txn,
            })
        } else {
            Ok(TransferResult {
                from_transaction: linked_txn,
                to_transaction: txn,
            })
        }
    }

    /// Get an active (non-archived) account or return an error
    fn get_active_account(&self, account_id: AccountId) -> EnvelopeResult<Account> {
        let account = self
            .storage
            .accounts
            .get(account_id)?
            .ok_or_else(|| EnvelopeError::account_not_found(account_id.to_string()))?;

        if account.archived {
            return Err(EnvelopeError::Validation(format!(
                "Account '{}' is archived and cannot be used for transfers",
                account.name
            )));
        }

        Ok(account)
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

    fn setup_test_accounts(storage: &Storage) -> (AccountId, AccountId) {
        let checking = Account::new("Checking", AccountType::Checking);
        let savings = Account::new("Savings", AccountType::Savings);

        let checking_id = checking.id;
        let savings_id = savings.id;

        storage.accounts.upsert(checking).unwrap();
        storage.accounts.upsert(savings).unwrap();
        storage.accounts.save().unwrap();

        (checking_id, savings_id)
    }

    #[test]
    fn test_create_transfer() {
        let (_temp_dir, storage) = create_test_storage();
        let (checking_id, savings_id) = setup_test_accounts(&storage);
        let service = TransferService::new(&storage);

        let result = service
            .create_transfer(
                checking_id,
                savings_id,
                Money::from_cents(50000),
                NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
                Some("Monthly savings".to_string()),
            )
            .unwrap();

        // Verify outflow from checking
        assert_eq!(result.from_transaction.account_id, checking_id);
        assert_eq!(result.from_transaction.amount.cents(), -50000);
        assert!(result.from_transaction.is_transfer());

        // Verify inflow to savings
        assert_eq!(result.to_transaction.account_id, savings_id);
        assert_eq!(result.to_transaction.amount.cents(), 50000);
        assert!(result.to_transaction.is_transfer());

        // Verify they're linked
        assert_eq!(
            result.from_transaction.transfer_transaction_id,
            Some(result.to_transaction.id)
        );
        assert_eq!(
            result.to_transaction.transfer_transaction_id,
            Some(result.from_transaction.id)
        );
    }

    #[test]
    fn test_transfer_to_same_account_fails() {
        let (_temp_dir, storage) = create_test_storage();
        let (checking_id, _) = setup_test_accounts(&storage);
        let service = TransferService::new(&storage);

        let result = service.create_transfer(
            checking_id,
            checking_id,
            Money::from_cents(50000),
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None,
        );

        assert!(matches!(result, Err(EnvelopeError::Validation(_))));
    }

    #[test]
    fn test_transfer_zero_amount_fails() {
        let (_temp_dir, storage) = create_test_storage();
        let (checking_id, savings_id) = setup_test_accounts(&storage);
        let service = TransferService::new(&storage);

        let result = service.create_transfer(
            checking_id,
            savings_id,
            Money::zero(),
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            None,
        );

        assert!(matches!(result, Err(EnvelopeError::Validation(_))));
    }

    #[test]
    fn test_update_transfer_amount() {
        let (_temp_dir, storage) = create_test_storage();
        let (checking_id, savings_id) = setup_test_accounts(&storage);
        let service = TransferService::new(&storage);

        let created = service
            .create_transfer(
                checking_id,
                savings_id,
                Money::from_cents(50000),
                NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
                None,
            )
            .unwrap();

        let updated = service
            .update_transfer_amount(created.from_transaction.id, Money::from_cents(75000))
            .unwrap();

        assert_eq!(updated.from_transaction.amount.cents(), -75000);
        assert_eq!(updated.to_transaction.amount.cents(), 75000);
    }

    #[test]
    fn test_update_transfer_date() {
        let (_temp_dir, storage) = create_test_storage();
        let (checking_id, savings_id) = setup_test_accounts(&storage);
        let service = TransferService::new(&storage);

        let created = service
            .create_transfer(
                checking_id,
                savings_id,
                Money::from_cents(50000),
                NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
                None,
            )
            .unwrap();

        let new_date = NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();
        let updated = service
            .update_transfer_date(created.from_transaction.id, new_date)
            .unwrap();

        assert_eq!(updated.from_transaction.date, new_date);
        assert_eq!(updated.to_transaction.date, new_date);
    }

    #[test]
    fn test_delete_transfer() {
        let (_temp_dir, storage) = create_test_storage();
        let (checking_id, savings_id) = setup_test_accounts(&storage);
        let service = TransferService::new(&storage);

        let created = service
            .create_transfer(
                checking_id,
                savings_id,
                Money::from_cents(50000),
                NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
                None,
            )
            .unwrap();

        assert_eq!(storage.transactions.count().unwrap(), 2);

        service
            .delete_transfer(created.from_transaction.id)
            .unwrap();

        assert_eq!(storage.transactions.count().unwrap(), 0);
    }

    #[test]
    fn test_get_linked_transaction() {
        let (_temp_dir, storage) = create_test_storage();
        let (checking_id, savings_id) = setup_test_accounts(&storage);
        let service = TransferService::new(&storage);

        let created = service
            .create_transfer(
                checking_id,
                savings_id,
                Money::from_cents(50000),
                NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
                None,
            )
            .unwrap();

        let linked = service
            .get_linked_transaction(created.from_transaction.id)
            .unwrap()
            .unwrap();

        assert_eq!(linked.id, created.to_transaction.id);
    }
}
