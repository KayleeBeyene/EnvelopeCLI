//! Transaction repository for JSON storage
//!
//! Manages loading and saving transactions to transactions.json

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use chrono::NaiveDate;

use crate::error::EnvelopeError;
use crate::models::{AccountId, CategoryId, Transaction, TransactionId};

use super::file_io::{read_json, write_json_atomic};

/// Serializable transaction data structure
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct TransactionData {
    transactions: Vec<Transaction>,
}

/// Repository for transaction persistence with indexing
pub struct TransactionRepository {
    path: PathBuf,
    data: RwLock<HashMap<TransactionId, Transaction>>,
    /// Index: account_id -> transaction_ids
    by_account: RwLock<HashMap<AccountId, Vec<TransactionId>>>,
    /// Index: category_id -> transaction_ids
    by_category: RwLock<HashMap<CategoryId, Vec<TransactionId>>>,
}

impl TransactionRepository {
    /// Create a new transaction repository
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            data: RwLock::new(HashMap::new()),
            by_account: RwLock::new(HashMap::new()),
            by_category: RwLock::new(HashMap::new()),
        }
    }

    /// Load transactions from disk and build indexes
    pub fn load(&self) -> Result<(), EnvelopeError> {
        let file_data: TransactionData = read_json(&self.path)?;

        let mut data = self.data.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;
        let mut by_account = self.by_account.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;
        let mut by_category = self.by_category.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;

        data.clear();
        by_account.clear();
        by_category.clear();

        for txn in file_data.transactions {
            let id = txn.id;
            let account_id = txn.account_id;

            // Index by account
            by_account.entry(account_id).or_default().push(id);

            // Index by category
            if let Some(cat_id) = txn.category_id {
                by_category.entry(cat_id).or_default().push(id);
            }
            for split in &txn.splits {
                by_category.entry(split.category_id).or_default().push(id);
            }

            data.insert(id, txn);
        }

        Ok(())
    }

    /// Save transactions to disk
    pub fn save(&self) -> Result<(), EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        let mut transactions: Vec<_> = data.values().cloned().collect();
        transactions.sort_by(|a, b| b.date.cmp(&a.date).then(b.created_at.cmp(&a.created_at)));

        let file_data = TransactionData { transactions };
        write_json_atomic(&self.path, &file_data)
    }

    /// Get a transaction by ID
    pub fn get(&self, id: TransactionId) -> Result<Option<Transaction>, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(data.get(&id).cloned())
    }

    /// Get all transactions
    pub fn get_all(&self) -> Result<Vec<Transaction>, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        let mut transactions: Vec<_> = data.values().cloned().collect();
        transactions.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(transactions)
    }

    /// Get transactions for an account
    pub fn get_by_account(&self, account_id: AccountId) -> Result<Vec<Transaction>, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;
        let by_account = self.by_account.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        let ids = by_account.get(&account_id).map(|v| v.as_slice()).unwrap_or(&[]);
        let mut transactions: Vec<_> = ids
            .iter()
            .filter_map(|id| data.get(id).cloned())
            .collect();
        transactions.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(transactions)
    }

    /// Get transactions for a category
    pub fn get_by_category(&self, category_id: CategoryId) -> Result<Vec<Transaction>, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;
        let by_category = self.by_category.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        let ids = by_category.get(&category_id).map(|v| v.as_slice()).unwrap_or(&[]);
        let mut transactions: Vec<_> = ids
            .iter()
            .filter_map(|id| data.get(id).cloned())
            .collect();
        transactions.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(transactions)
    }

    /// Get transactions in a date range
    pub fn get_by_date_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Transaction>, EnvelopeError> {
        let all = self.get_all()?;
        Ok(all
            .into_iter()
            .filter(|t| t.date >= start && t.date <= end)
            .collect())
    }

    /// Insert or update a transaction
    pub fn upsert(&self, txn: Transaction) -> Result<(), EnvelopeError> {
        let mut data = self.data.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;
        let mut by_account = self.by_account.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;
        let mut by_category = self.by_category.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;

        // Remove from old indexes if updating
        if let Some(old) = data.get(&txn.id) {
            if let Some(ids) = by_account.get_mut(&old.account_id) {
                ids.retain(|&id| id != txn.id);
            }
            if let Some(cat_id) = old.category_id {
                if let Some(ids) = by_category.get_mut(&cat_id) {
                    ids.retain(|&id| id != txn.id);
                }
            }
            for split in &old.splits {
                if let Some(ids) = by_category.get_mut(&split.category_id) {
                    ids.retain(|&id| id != txn.id);
                }
            }
        }

        // Add to new indexes
        by_account.entry(txn.account_id).or_default().push(txn.id);
        if let Some(cat_id) = txn.category_id {
            by_category.entry(cat_id).or_default().push(txn.id);
        }
        for split in &txn.splits {
            by_category.entry(split.category_id).or_default().push(txn.id);
        }

        data.insert(txn.id, txn);
        Ok(())
    }

    /// Delete a transaction
    pub fn delete(&self, id: TransactionId) -> Result<bool, EnvelopeError> {
        let mut data = self.data.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;
        let mut by_account = self.by_account.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;
        let mut by_category = self.by_category.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;

        if let Some(txn) = data.remove(&id) {
            // Remove from indexes
            if let Some(ids) = by_account.get_mut(&txn.account_id) {
                ids.retain(|&tid| tid != id);
            }
            if let Some(cat_id) = txn.category_id {
                if let Some(ids) = by_category.get_mut(&cat_id) {
                    ids.retain(|&tid| tid != id);
                }
            }
            for split in &txn.splits {
                if let Some(ids) = by_category.get_mut(&split.category_id) {
                    ids.retain(|&tid| tid != id);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Find transaction by import ID
    pub fn find_by_import_id(&self, import_id: &str) -> Result<Option<Transaction>, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(data
            .values()
            .find(|t| t.import_id.as_deref() == Some(import_id))
            .cloned())
    }

    /// Count transactions
    pub fn count(&self) -> Result<usize, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(data.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Money;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, TransactionRepository) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("transactions.json");
        let repo = TransactionRepository::new(path);
        (temp_dir, repo)
    }

    #[test]
    fn test_empty_load() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();
        assert_eq!(repo.count().unwrap(), 0);
    }

    #[test]
    fn test_upsert_and_get() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let account_id = AccountId::new();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let txn = Transaction::new(account_id, date, Money::from_cents(-5000));
        let id = txn.id;

        repo.upsert(txn).unwrap();

        let retrieved = repo.get(id).unwrap().unwrap();
        assert_eq!(retrieved.amount.cents(), -5000);
    }

    #[test]
    fn test_get_by_account() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let account1 = AccountId::new();
        let account2 = AccountId::new();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        repo.upsert(Transaction::new(account1, date, Money::from_cents(-100))).unwrap();
        repo.upsert(Transaction::new(account1, date, Money::from_cents(-200))).unwrap();
        repo.upsert(Transaction::new(account2, date, Money::from_cents(-300))).unwrap();

        let account1_txns = repo.get_by_account(account1).unwrap();
        assert_eq!(account1_txns.len(), 2);

        let account2_txns = repo.get_by_account(account2).unwrap();
        assert_eq!(account2_txns.len(), 1);
    }

    #[test]
    fn test_save_and_reload() {
        let (temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let account_id = AccountId::new();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let txn = Transaction::new(account_id, date, Money::from_cents(-5000));
        let id = txn.id;

        repo.upsert(txn).unwrap();
        repo.save().unwrap();

        // Create new repo and load
        let path = temp_dir.path().join("transactions.json");
        let repo2 = TransactionRepository::new(path);
        repo2.load().unwrap();

        assert_eq!(repo2.count().unwrap(), 1);
        let retrieved = repo2.get(id).unwrap().unwrap();
        assert_eq!(retrieved.amount.cents(), -5000);
    }

    #[test]
    fn test_delete() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let account_id = AccountId::new();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let txn = Transaction::new(account_id, date, Money::from_cents(-5000));
        let id = txn.id;

        repo.upsert(txn).unwrap();
        assert_eq!(repo.count().unwrap(), 1);

        repo.delete(id).unwrap();
        assert_eq!(repo.count().unwrap(), 0);
    }

    #[test]
    fn test_date_range_query() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let account_id = AccountId::new();
        repo.upsert(Transaction::new(
            account_id,
            NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            Money::from_cents(-100),
        )).unwrap();
        repo.upsert(Transaction::new(
            account_id,
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            Money::from_cents(-200),
        )).unwrap();
        repo.upsert(Transaction::new(
            account_id,
            NaiveDate::from_ymd_opt(2025, 1, 20).unwrap(),
            Money::from_cents(-300),
        )).unwrap();

        let range = repo.get_by_date_range(
            NaiveDate::from_ymd_opt(2025, 1, 12).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 18).unwrap(),
        ).unwrap();

        assert_eq!(range.len(), 1);
        assert_eq!(range[0].amount.cents(), -200);
    }
}
