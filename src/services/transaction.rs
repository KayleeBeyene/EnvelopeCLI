//! Transaction service
//!
//! Provides business logic for transaction management including CRUD operations,
//! status management, and integration with budget calculations.

use chrono::{NaiveDate, Utc};

use crate::audit::EntityType;
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{
    AccountId, CategoryId, Money, Split, Transaction, TransactionId, TransactionStatus,
};
use crate::storage::Storage;

/// Service for transaction management
pub struct TransactionService<'a> {
    storage: &'a Storage,
}

/// Options for filtering transactions
#[derive(Debug, Clone, Default)]
pub struct TransactionFilter {
    /// Filter by account
    pub account_id: Option<AccountId>,
    /// Filter by category
    pub category_id: Option<CategoryId>,
    /// Filter by date range start
    pub start_date: Option<NaiveDate>,
    /// Filter by date range end
    pub end_date: Option<NaiveDate>,
    /// Filter by status
    pub status: Option<TransactionStatus>,
    /// Maximum number of transactions to return
    pub limit: Option<usize>,
}

impl TransactionFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by account
    pub fn account(mut self, account_id: AccountId) -> Self {
        self.account_id = Some(account_id);
        self
    }

    /// Filter by category
    pub fn category(mut self, category_id: CategoryId) -> Self {
        self.category_id = Some(category_id);
        self
    }

    /// Filter by date range
    pub fn date_range(mut self, start: NaiveDate, end: NaiveDate) -> Self {
        self.start_date = Some(start);
        self.end_date = Some(end);
        self
    }

    /// Filter by status
    pub fn status(mut self, status: TransactionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Limit results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Input for creating a new transaction
#[derive(Debug, Clone)]
pub struct CreateTransactionInput {
    pub account_id: AccountId,
    pub date: NaiveDate,
    pub amount: Money,
    pub payee_name: Option<String>,
    pub category_id: Option<CategoryId>,
    pub memo: Option<String>,
    pub status: Option<TransactionStatus>,
}

impl<'a> TransactionService<'a> {
    /// Create a new transaction service
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Create a new transaction
    pub fn create(&self, input: CreateTransactionInput) -> EnvelopeResult<Transaction> {
        // Verify account exists
        let account = self
            .storage
            .accounts
            .get(input.account_id)?
            .ok_or_else(|| EnvelopeError::account_not_found(input.account_id.to_string()))?;

        if account.archived {
            return Err(EnvelopeError::Validation(
                "Cannot add transactions to an archived account".into(),
            ));
        }

        // Verify category exists if provided
        if let Some(cat_id) = input.category_id {
            self.storage
                .categories
                .get_category(cat_id)?
                .ok_or_else(|| EnvelopeError::category_not_found(cat_id.to_string()))?;
        }

        // Create the transaction
        let mut txn = Transaction::new(input.account_id, input.date, input.amount);

        if let Some(payee_name) = input.payee_name {
            txn.payee_name = payee_name.trim().to_string();

            // Try to find or create payee
            if !txn.payee_name.is_empty() {
                let payee = self.storage.payees.get_or_create(&txn.payee_name)?;
                txn.payee_id = Some(payee.id);
            }
        }

        txn.category_id = input.category_id;

        if let Some(memo) = input.memo {
            txn.memo = memo;
        }

        if let Some(status) = input.status {
            txn.status = status;
        }

        // Validate
        txn.validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        // Save transaction
        self.storage.transactions.upsert(txn.clone())?;
        self.storage.transactions.save()?;

        // Save payees if modified
        self.storage.payees.save()?;

        // Audit log
        self.storage.log_create(
            EntityType::Transaction,
            txn.id.to_string(),
            Some(format!("{} {}", txn.date, txn.payee_name)),
            &txn,
        )?;

        Ok(txn)
    }

    /// Get a transaction by ID
    pub fn get(&self, id: TransactionId) -> EnvelopeResult<Option<Transaction>> {
        self.storage.transactions.get(id)
    }

    /// Find a transaction by ID string
    pub fn find(&self, identifier: &str) -> EnvelopeResult<Option<Transaction>> {
        if let Ok(id) = identifier.parse::<TransactionId>() {
            return self.storage.transactions.get(id);
        }
        Ok(None)
    }

    /// List all transactions with optional filtering
    pub fn list(&self, filter: TransactionFilter) -> EnvelopeResult<Vec<Transaction>> {
        let mut transactions = if let Some(account_id) = filter.account_id {
            self.storage.transactions.get_by_account(account_id)?
        } else if let Some(category_id) = filter.category_id {
            self.storage.transactions.get_by_category(category_id)?
        } else if let (Some(start), Some(end)) = (filter.start_date, filter.end_date) {
            self.storage.transactions.get_by_date_range(start, end)?
        } else {
            self.storage.transactions.get_all()?
        };

        // Apply additional filters
        if let Some(start) = filter.start_date {
            transactions.retain(|t| t.date >= start);
        }
        if let Some(end) = filter.end_date {
            transactions.retain(|t| t.date <= end);
        }
        if let Some(status) = filter.status {
            transactions.retain(|t| t.status == status);
        }

        // Apply limit
        if let Some(limit) = filter.limit {
            transactions.truncate(limit);
        }

        Ok(transactions)
    }

    /// Get transactions for an account
    pub fn list_for_account(&self, account_id: AccountId) -> EnvelopeResult<Vec<Transaction>> {
        self.storage.transactions.get_by_account(account_id)
    }

    /// Get transactions for a category
    pub fn list_for_category(&self, category_id: CategoryId) -> EnvelopeResult<Vec<Transaction>> {
        self.storage.transactions.get_by_category(category_id)
    }

    /// Update a transaction
    pub fn update(
        &self,
        id: TransactionId,
        date: Option<NaiveDate>,
        amount: Option<Money>,
        payee_name: Option<String>,
        category_id: Option<Option<CategoryId>>,
        memo: Option<String>,
    ) -> EnvelopeResult<Transaction> {
        let mut txn = self
            .storage
            .transactions
            .get(id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(id.to_string()))?;

        // Check if locked
        if txn.is_locked() {
            return Err(EnvelopeError::Locked(format!(
                "Transaction {} is reconciled and cannot be edited. Unlock it first.",
                id
            )));
        }

        let before = txn.clone();

        // Apply updates
        if let Some(new_date) = date {
            txn.date = new_date;
        }

        if let Some(new_amount) = amount {
            txn.amount = new_amount;
        }

        if let Some(new_payee_name) = payee_name {
            txn.payee_name = new_payee_name.trim().to_string();
            if !txn.payee_name.is_empty() {
                let payee = self.storage.payees.get_or_create(&txn.payee_name)?;
                txn.payee_id = Some(payee.id);
            } else {
                txn.payee_id = None;
            }
        }

        // category_id: Option<Option<CategoryId>>
        // - None: no change
        // - Some(None): clear category
        // - Some(Some(id)): set category
        if let Some(new_cat_id) = category_id {
            if let Some(cat_id) = new_cat_id {
                // Verify category exists
                self.storage
                    .categories
                    .get_category(cat_id)?
                    .ok_or_else(|| EnvelopeError::category_not_found(cat_id.to_string()))?;
            }
            txn.category_id = new_cat_id;
            // Clear splits if setting a category
            if new_cat_id.is_some() {
                txn.splits.clear();
            }
        }

        if let Some(new_memo) = memo {
            txn.memo = new_memo;
        }

        txn.updated_at = Utc::now();

        // Validate
        txn.validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        // Save
        self.storage.transactions.upsert(txn.clone())?;
        self.storage.transactions.save()?;
        self.storage.payees.save()?;

        // Build diff summary
        let mut changes = Vec::new();
        if before.date != txn.date {
            changes.push(format!("date: {} -> {}", before.date, txn.date));
        }
        if before.amount != txn.amount {
            changes.push(format!("amount: {} -> {}", before.amount, txn.amount));
        }
        if before.payee_name != txn.payee_name {
            changes.push(format!(
                "payee: '{}' -> '{}'",
                before.payee_name, txn.payee_name
            ));
        }
        if before.category_id != txn.category_id {
            changes.push(format!(
                "category: {:?} -> {:?}",
                before.category_id, txn.category_id
            ));
        }
        if before.memo != txn.memo {
            changes.push("memo changed".to_string());
        }

        let diff = if changes.is_empty() {
            None
        } else {
            Some(changes.join(", "))
        };

        // Audit log
        self.storage.log_update(
            EntityType::Transaction,
            txn.id.to_string(),
            Some(format!("{} {}", txn.date, txn.payee_name)),
            &before,
            &txn,
            diff,
        )?;

        Ok(txn)
    }

    /// Delete a transaction
    pub fn delete(&self, id: TransactionId) -> EnvelopeResult<Transaction> {
        let txn = self
            .storage
            .transactions
            .get(id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(id.to_string()))?;

        // Check if locked
        if txn.is_locked() {
            return Err(EnvelopeError::Locked(format!(
                "Transaction {} is reconciled and cannot be deleted. Unlock it first.",
                id
            )));
        }

        // If this is a transfer, we need to handle the linked transaction
        if let Some(linked_id) = txn.transfer_transaction_id {
            // Delete the linked transaction too
            if let Some(linked_txn) = self.storage.transactions.get(linked_id)? {
                if linked_txn.is_locked() {
                    return Err(EnvelopeError::Locked(format!(
                        "Linked transfer transaction {} is reconciled and cannot be deleted.",
                        linked_id
                    )));
                }
                self.storage.transactions.delete(linked_id)?;
                self.storage.log_delete(
                    EntityType::Transaction,
                    linked_id.to_string(),
                    Some(format!(
                        "{} {} (linked)",
                        linked_txn.date, linked_txn.payee_name
                    )),
                    &linked_txn,
                )?;
            }
        }

        // Delete the transaction
        self.storage.transactions.delete(id)?;
        self.storage.transactions.save()?;

        // Audit log
        self.storage.log_delete(
            EntityType::Transaction,
            id.to_string(),
            Some(format!("{} {}", txn.date, txn.payee_name)),
            &txn,
        )?;

        Ok(txn)
    }

    /// Set the status of a transaction
    pub fn set_status(
        &self,
        id: TransactionId,
        status: TransactionStatus,
    ) -> EnvelopeResult<Transaction> {
        let mut txn = self
            .storage
            .transactions
            .get(id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(id.to_string()))?;

        // Can't change status of reconciled transaction without unlocking first
        if txn.is_locked() && status != TransactionStatus::Reconciled {
            return Err(EnvelopeError::Locked(format!(
                "Transaction {} is reconciled. Unlock it before changing status.",
                id
            )));
        }

        let before = txn.clone();
        txn.set_status(status);

        // Save
        self.storage.transactions.upsert(txn.clone())?;
        self.storage.transactions.save()?;

        // Audit log
        self.storage.log_update(
            EntityType::Transaction,
            txn.id.to_string(),
            Some(format!("{} {}", txn.date, txn.payee_name)),
            &before,
            &txn,
            Some(format!("status: {} -> {}", before.status, txn.status)),
        )?;

        Ok(txn)
    }

    /// Clear a transaction (mark as cleared)
    pub fn clear(&self, id: TransactionId) -> EnvelopeResult<Transaction> {
        self.set_status(id, TransactionStatus::Cleared)
    }

    /// Unclear a transaction (mark as pending)
    pub fn unclear(&self, id: TransactionId) -> EnvelopeResult<Transaction> {
        self.set_status(id, TransactionStatus::Pending)
    }

    /// Unlock a reconciled transaction for editing
    ///
    /// This is a potentially dangerous operation - it allows editing a transaction
    /// that has already been reconciled with a bank statement. Use with caution.
    pub fn unlock(&self, id: TransactionId) -> EnvelopeResult<Transaction> {
        let mut txn = self
            .storage
            .transactions
            .get(id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(id.to_string()))?;

        if !txn.is_locked() {
            return Err(EnvelopeError::Validation(format!(
                "Transaction {} is not locked",
                id
            )));
        }

        let before = txn.clone();
        txn.set_status(TransactionStatus::Cleared);

        // Save
        self.storage.transactions.upsert(txn.clone())?;
        self.storage.transactions.save()?;

        // Audit log - this is important to track
        self.storage.log_update(
            EntityType::Transaction,
            txn.id.to_string(),
            Some(format!("{} {}", txn.date, txn.payee_name)),
            &before,
            &txn,
            Some("UNLOCKED: reconciled -> cleared".to_string()),
        )?;

        Ok(txn)
    }

    /// Add a split to a transaction
    ///
    /// Note: This validates that splits total equals the transaction amount.
    /// If you need to add multiple splits, use `set_splits` instead to avoid
    /// intermediate validation failures.
    pub fn add_split(
        &self,
        id: TransactionId,
        category_id: CategoryId,
        amount: Money,
        memo: Option<String>,
    ) -> EnvelopeResult<Transaction> {
        let mut txn = self
            .storage
            .transactions
            .get(id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(id.to_string()))?;

        if txn.is_locked() {
            return Err(EnvelopeError::Locked(format!(
                "Transaction {} is reconciled and cannot be edited.",
                id
            )));
        }

        // Verify category exists
        self.storage
            .categories
            .get_category(category_id)?
            .ok_or_else(|| EnvelopeError::category_not_found(category_id.to_string()))?;

        let before = txn.clone();

        // Add the split
        let split = if let Some(memo) = memo {
            Split::with_memo(category_id, amount, memo)
        } else {
            Split::new(category_id, amount)
        };
        txn.add_split(split);

        // Validate
        txn.validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        // Save
        self.storage.transactions.upsert(txn.clone())?;
        self.storage.transactions.save()?;

        // Audit log
        self.storage.log_update(
            EntityType::Transaction,
            txn.id.to_string(),
            Some(format!("{} {}", txn.date, txn.payee_name)),
            &before,
            &txn,
            Some(format!(
                "added split: {} to category {}",
                amount, category_id
            )),
        )?;

        Ok(txn)
    }

    /// Set all splits for a transaction at once
    ///
    /// This replaces any existing splits with the new ones.
    /// The splits must sum to the transaction amount.
    pub fn set_splits(&self, id: TransactionId, splits: Vec<Split>) -> EnvelopeResult<Transaction> {
        let mut txn = self
            .storage
            .transactions
            .get(id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(id.to_string()))?;

        if txn.is_locked() {
            return Err(EnvelopeError::Locked(format!(
                "Transaction {} is reconciled and cannot be edited.",
                id
            )));
        }

        // Verify all categories exist
        for split in &splits {
            self.storage
                .categories
                .get_category(split.category_id)?
                .ok_or_else(|| EnvelopeError::category_not_found(split.category_id.to_string()))?;
        }

        let before = txn.clone();

        // Replace splits
        txn.splits = splits;
        txn.category_id = None; // Clear single category when using splits
        txn.updated_at = Utc::now();

        // Validate
        txn.validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        // Save
        self.storage.transactions.upsert(txn.clone())?;
        self.storage.transactions.save()?;

        // Audit log
        self.storage.log_update(
            EntityType::Transaction,
            txn.id.to_string(),
            Some(format!("{} {}", txn.date, txn.payee_name)),
            &before,
            &txn,
            Some(format!("set {} splits", txn.splits.len())),
        )?;

        Ok(txn)
    }

    /// Clear all splits from a transaction
    pub fn clear_splits(&self, id: TransactionId) -> EnvelopeResult<Transaction> {
        let mut txn = self
            .storage
            .transactions
            .get(id)?
            .ok_or_else(|| EnvelopeError::transaction_not_found(id.to_string()))?;

        if txn.is_locked() {
            return Err(EnvelopeError::Locked(format!(
                "Transaction {} is reconciled and cannot be edited.",
                id
            )));
        }

        if txn.splits.is_empty() {
            return Ok(txn);
        }

        let before = txn.clone();
        txn.splits.clear();
        txn.updated_at = Utc::now();

        // Save
        self.storage.transactions.upsert(txn.clone())?;
        self.storage.transactions.save()?;

        // Audit log
        self.storage.log_update(
            EntityType::Transaction,
            txn.id.to_string(),
            Some(format!("{} {}", txn.date, txn.payee_name)),
            &before,
            &txn,
            Some("cleared all splits".to_string()),
        )?;

        Ok(txn)
    }

    /// Learn from a transaction - update payee's category frequency
    pub fn learn_from_transaction(&self, txn: &Transaction) -> EnvelopeResult<()> {
        if let (Some(payee_id), Some(category_id)) = (txn.payee_id, txn.category_id) {
            if let Some(mut payee) = self.storage.payees.get(payee_id)? {
                payee.record_category_usage(category_id);
                self.storage.payees.upsert(payee)?;
                self.storage.payees.save()?;
            }
        }
        Ok(())
    }

    /// Get suggested category for a payee name
    pub fn suggest_category(&self, payee_name: &str) -> EnvelopeResult<Option<CategoryId>> {
        if let Some(payee) = self.storage.payees.get_by_name(payee_name)? {
            Ok(payee.suggested_category())
        } else {
            Ok(None)
        }
    }

    /// Count transactions
    pub fn count(&self) -> EnvelopeResult<usize> {
        self.storage.transactions.count()
    }

    /// Get uncleared transactions for an account
    pub fn get_uncleared(&self, account_id: AccountId) -> EnvelopeResult<Vec<Transaction>> {
        let transactions = self.storage.transactions.get_by_account(account_id)?;
        Ok(transactions
            .into_iter()
            .filter(|t| t.status == TransactionStatus::Pending)
            .collect())
    }

    /// Get cleared (but not reconciled) transactions for an account
    pub fn get_cleared(&self, account_id: AccountId) -> EnvelopeResult<Vec<Transaction>> {
        let transactions = self.storage.transactions.get_by_account(account_id)?;
        Ok(transactions
            .into_iter()
            .filter(|t| t.status == TransactionStatus::Cleared)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::EnvelopePaths;
    use crate::models::{Account, AccountType, Category, CategoryGroup};
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, Storage) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        let mut storage = Storage::new(paths).unwrap();
        storage.load_all().unwrap();
        (temp_dir, storage)
    }

    fn setup_test_data(storage: &Storage) -> (AccountId, CategoryId) {
        // Create an account
        let account = Account::new("Checking", AccountType::Checking);
        let account_id = account.id;
        storage.accounts.upsert(account).unwrap();
        storage.accounts.save().unwrap();

        // Create a category
        let group = CategoryGroup::new("Test Group");
        storage.categories.upsert_group(group.clone()).unwrap();

        let category = Category::new("Groceries", group.id);
        let category_id = category.id;
        storage.categories.upsert_category(category).unwrap();
        storage.categories.save().unwrap();

        (account_id, category_id)
    }

    #[test]
    fn test_create_transaction() {
        let (_temp_dir, storage) = create_test_storage();
        let (account_id, category_id) = setup_test_data(&storage);
        let service = TransactionService::new(&storage);

        let input = CreateTransactionInput {
            account_id,
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            amount: Money::from_cents(-5000),
            payee_name: Some("Test Store".to_string()),
            category_id: Some(category_id),
            memo: Some("Test purchase".to_string()),
            status: None,
        };

        let txn = service.create(input).unwrap();

        assert_eq!(txn.amount.cents(), -5000);
        assert_eq!(txn.payee_name, "Test Store");
        assert_eq!(txn.category_id, Some(category_id));
        assert_eq!(txn.status, TransactionStatus::Pending);
    }

    #[test]
    fn test_list_transactions() {
        let (_temp_dir, storage) = create_test_storage();
        let (account_id, category_id) = setup_test_data(&storage);
        let service = TransactionService::new(&storage);

        // Create a few transactions
        for i in 1..=3 {
            let input = CreateTransactionInput {
                account_id,
                date: NaiveDate::from_ymd_opt(2025, 1, i as u32).unwrap(),
                amount: Money::from_cents(-1000 * i),
                payee_name: Some(format!("Store {}", i)),
                category_id: Some(category_id),
                memo: None,
                status: None,
            };
            service.create(input).unwrap();
        }

        let transactions = service.list(TransactionFilter::new()).unwrap();
        assert_eq!(transactions.len(), 3);

        // Filter by account
        let filtered = service
            .list(TransactionFilter::new().account(account_id))
            .unwrap();
        assert_eq!(filtered.len(), 3);

        // Limit results
        let limited = service.list(TransactionFilter::new().limit(2)).unwrap();
        assert_eq!(limited.len(), 2);
    }

    #[test]
    fn test_update_transaction() {
        let (_temp_dir, storage) = create_test_storage();
        let (account_id, _category_id) = setup_test_data(&storage);
        let service = TransactionService::new(&storage);

        let input = CreateTransactionInput {
            account_id,
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            amount: Money::from_cents(-5000),
            payee_name: Some("Original Store".to_string()),
            category_id: None,
            memo: None,
            status: None,
        };

        let txn = service.create(input).unwrap();

        // Update the transaction
        let updated = service
            .update(
                txn.id,
                None,
                Some(Money::from_cents(-7500)),
                Some("Updated Store".to_string()),
                None,
                Some("Updated memo".to_string()),
            )
            .unwrap();

        assert_eq!(updated.amount.cents(), -7500);
        assert_eq!(updated.payee_name, "Updated Store");
        assert_eq!(updated.memo, "Updated memo");
    }

    #[test]
    fn test_delete_transaction() {
        let (_temp_dir, storage) = create_test_storage();
        let (account_id, _category_id) = setup_test_data(&storage);
        let service = TransactionService::new(&storage);

        let input = CreateTransactionInput {
            account_id,
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            amount: Money::from_cents(-5000),
            payee_name: None,
            category_id: None,
            memo: None,
            status: None,
        };

        let txn = service.create(input).unwrap();
        assert_eq!(service.count().unwrap(), 1);

        service.delete(txn.id).unwrap();
        assert_eq!(service.count().unwrap(), 0);
    }

    #[test]
    fn test_status_transitions() {
        let (_temp_dir, storage) = create_test_storage();
        let (account_id, _category_id) = setup_test_data(&storage);
        let service = TransactionService::new(&storage);

        let input = CreateTransactionInput {
            account_id,
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            amount: Money::from_cents(-5000),
            payee_name: None,
            category_id: None,
            memo: None,
            status: None,
        };

        let txn = service.create(input).unwrap();
        assert_eq!(txn.status, TransactionStatus::Pending);

        // Clear
        let cleared = service.clear(txn.id).unwrap();
        assert_eq!(cleared.status, TransactionStatus::Cleared);

        // Unclear
        let uncleared = service.unclear(txn.id).unwrap();
        assert_eq!(uncleared.status, TransactionStatus::Pending);
    }

    #[test]
    fn test_locked_transaction() {
        let (_temp_dir, storage) = create_test_storage();
        let (account_id, _category_id) = setup_test_data(&storage);
        let service = TransactionService::new(&storage);

        let input = CreateTransactionInput {
            account_id,
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            amount: Money::from_cents(-5000),
            payee_name: None,
            category_id: None,
            memo: None,
            status: None,
        };

        let txn = service.create(input).unwrap();

        // Reconcile (lock)
        let reconciled = service
            .set_status(txn.id, TransactionStatus::Reconciled)
            .unwrap();
        assert!(reconciled.is_locked());

        // Try to update - should fail
        let update_result = service.update(
            txn.id,
            None,
            Some(Money::from_cents(-7500)),
            None,
            None,
            None,
        );
        assert!(matches!(update_result, Err(EnvelopeError::Locked(_))));

        // Try to delete - should fail
        let delete_result = service.delete(txn.id);
        assert!(matches!(delete_result, Err(EnvelopeError::Locked(_))));

        // Unlock
        let unlocked = service.unlock(txn.id).unwrap();
        assert!(!unlocked.is_locked());

        // Now update should work
        let updated = service
            .update(
                txn.id,
                None,
                Some(Money::from_cents(-7500)),
                None,
                None,
                None,
            )
            .unwrap();
        assert_eq!(updated.amount.cents(), -7500);
    }

    #[test]
    fn test_split_transactions() {
        let (_temp_dir, storage) = create_test_storage();
        let (account_id, category_id) = setup_test_data(&storage);
        let service = TransactionService::new(&storage);

        // Create another category for split
        let category2 = Category::new(
            "Household",
            storage
                .categories
                .get_all_groups()
                .unwrap()
                .first()
                .unwrap()
                .id,
        );
        let category2_id = category2.id;
        storage.categories.upsert_category(category2).unwrap();
        storage.categories.save().unwrap();

        let input = CreateTransactionInput {
            account_id,
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            amount: Money::from_cents(-10000),
            payee_name: Some("Multi-Store".to_string()),
            category_id: None,
            memo: None,
            status: None,
        };

        let txn = service.create(input).unwrap();

        // Set splits using set_splits to add multiple splits at once
        let splits = vec![
            Split::new(category_id, Money::from_cents(-6000)),
            Split::with_memo(
                category2_id,
                Money::from_cents(-4000),
                "Cleaning supplies".to_string(),
            ),
        ];

        let final_txn = service.set_splits(txn.id, splits).unwrap();

        assert!(final_txn.is_split());
        assert_eq!(final_txn.splits.len(), 2);
        assert!(final_txn.validate().is_ok());
    }
}
