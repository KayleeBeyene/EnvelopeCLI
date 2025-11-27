//! Account repository for JSON storage
//!
//! Manages loading and saving accounts to accounts.json

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::error::EnvelopeError;
use crate::models::{Account, AccountId};

use super::file_io::{read_json, write_json_atomic};

/// Serializable account data structure
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct AccountData {
    accounts: Vec<Account>,
}

/// Repository for account persistence
pub struct AccountRepository {
    path: PathBuf,
    data: RwLock<HashMap<AccountId, Account>>,
}

impl AccountRepository {
    /// Create a new account repository
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            data: RwLock::new(HashMap::new()),
        }
    }

    /// Load accounts from disk
    pub fn load(&self) -> Result<(), EnvelopeError> {
        let file_data: AccountData = read_json(&self.path)?;

        let mut data = self.data.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;

        data.clear();
        for account in file_data.accounts {
            data.insert(account.id, account);
        }

        Ok(())
    }

    /// Save accounts to disk
    pub fn save(&self) -> Result<(), EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        let file_data = AccountData {
            accounts: data.values().cloned().collect(),
        };

        write_json_atomic(&self.path, &file_data)
    }

    /// Get an account by ID
    pub fn get(&self, id: AccountId) -> Result<Option<Account>, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(data.get(&id).cloned())
    }

    /// Get all accounts
    pub fn get_all(&self) -> Result<Vec<Account>, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        let mut accounts: Vec<_> = data.values().cloned().collect();
        accounts.sort_by(|a, b| a.sort_order.cmp(&b.sort_order).then(a.name.cmp(&b.name)));
        Ok(accounts)
    }

    /// Get all active (non-archived) accounts
    pub fn get_active(&self) -> Result<Vec<Account>, EnvelopeError> {
        let all = self.get_all()?;
        Ok(all.into_iter().filter(|a| !a.archived).collect())
    }

    /// Get an account by name (case-insensitive)
    pub fn get_by_name(&self, name: &str) -> Result<Option<Account>, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        let name_lower = name.to_lowercase();
        Ok(data
            .values()
            .find(|a| a.name.to_lowercase() == name_lower)
            .cloned())
    }

    /// Insert or update an account
    pub fn upsert(&self, account: Account) -> Result<(), EnvelopeError> {
        let mut data = self.data.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;

        data.insert(account.id, account);
        Ok(())
    }

    /// Delete an account
    pub fn delete(&self, id: AccountId) -> Result<bool, EnvelopeError> {
        let mut data = self.data.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;

        Ok(data.remove(&id).is_some())
    }

    /// Check if an account exists
    pub fn exists(&self, id: AccountId) -> Result<bool, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(data.contains_key(&id))
    }

    /// Check if an account name is already taken
    pub fn name_exists(&self, name: &str, exclude_id: Option<AccountId>) -> Result<bool, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        let name_lower = name.to_lowercase();
        Ok(data.values().any(|a| {
            a.name.to_lowercase() == name_lower && Some(a.id) != exclude_id
        }))
    }

    /// Count accounts
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
    use crate::models::AccountType;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, AccountRepository) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("accounts.json");
        let repo = AccountRepository::new(path);
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

        let account = Account::new("Checking", AccountType::Checking);
        let id = account.id;

        repo.upsert(account.clone()).unwrap();

        let retrieved = repo.get(id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Checking");
    }

    #[test]
    fn test_save_and_reload() {
        let (temp_dir, repo) = create_test_repo();

        let account = Account::new("Savings", AccountType::Savings);
        let id = account.id;

        repo.load().unwrap();
        repo.upsert(account).unwrap();
        repo.save().unwrap();

        // Create new repo and load
        let path = temp_dir.path().join("accounts.json");
        let repo2 = AccountRepository::new(path);
        repo2.load().unwrap();

        let retrieved = repo2.get(id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Savings");
    }

    #[test]
    fn test_get_by_name() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let account = Account::new("My Checking", AccountType::Checking);
        repo.upsert(account).unwrap();

        // Case insensitive
        let found = repo.get_by_name("my checking").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "My Checking");

        let not_found = repo.get_by_name("other").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_delete() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let account = Account::new("Test", AccountType::Checking);
        let id = account.id;

        repo.upsert(account).unwrap();
        assert!(repo.exists(id).unwrap());

        repo.delete(id).unwrap();
        assert!(!repo.exists(id).unwrap());
    }

    #[test]
    fn test_get_active_filters_archived() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let account1 = Account::new("Active", AccountType::Checking);
        let mut account2 = Account::new("Archived", AccountType::Savings);
        account2.archive();

        repo.upsert(account1).unwrap();
        repo.upsert(account2).unwrap();

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 2);

        let active = repo.get_active().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "Active");
    }

    #[test]
    fn test_name_exists() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let account = Account::new("Test Account", AccountType::Checking);
        let id = account.id;
        repo.upsert(account).unwrap();

        // Name exists
        assert!(repo.name_exists("test account", None).unwrap());

        // Exclude self
        assert!(!repo.name_exists("test account", Some(id)).unwrap());

        // Different name
        assert!(!repo.name_exists("other", None).unwrap());
    }
}
