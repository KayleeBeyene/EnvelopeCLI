//! Storage layer for EnvelopeCLI
//!
//! Provides JSON file storage with atomic writes, file locking, and
//! automatic directory creation.

pub mod accounts;
pub mod budget;
pub mod categories;
pub mod file_io;
pub mod init;
pub mod payees;
pub mod transactions;

pub use accounts::AccountRepository;
pub use budget::BudgetRepository;
pub use categories::CategoryRepository;
pub use file_io::{read_json, write_json_atomic};
pub use init::initialize_storage;
pub use payees::PayeeRepository;
pub use transactions::TransactionRepository;

use crate::config::paths::EnvelopePaths;
use crate::error::EnvelopeError;

/// Main storage coordinator that provides access to all repositories
pub struct Storage {
    paths: EnvelopePaths,
    pub accounts: AccountRepository,
    pub transactions: TransactionRepository,
    pub categories: CategoryRepository,
    pub budget: BudgetRepository,
    pub payees: PayeeRepository,
}

impl Storage {
    /// Create a new Storage instance
    pub fn new(paths: EnvelopePaths) -> Result<Self, EnvelopeError> {
        // Ensure directories exist
        paths.ensure_directories()?;

        Ok(Self {
            accounts: AccountRepository::new(paths.accounts_file()),
            transactions: TransactionRepository::new(paths.transactions_file()),
            categories: CategoryRepository::new(paths.budget_file()),
            budget: BudgetRepository::new(paths.budget_file()),
            payees: PayeeRepository::new(paths.payees_file()),
            paths,
        })
    }

    /// Get the paths configuration
    pub fn paths(&self) -> &EnvelopePaths {
        &self.paths
    }

    /// Load all data from disk
    pub fn load_all(&mut self) -> Result<(), EnvelopeError> {
        self.accounts.load()?;
        self.transactions.load()?;
        self.categories.load()?;
        self.budget.load()?;
        self.payees.load()?;
        Ok(())
    }

    /// Save all data to disk
    pub fn save_all(&self) -> Result<(), EnvelopeError> {
        self.accounts.save()?;
        self.transactions.save()?;
        self.categories.save()?;
        self.budget.save()?;
        self.payees.save()?;
        Ok(())
    }

    /// Check if storage has been initialized (has any data)
    pub fn is_initialized(&self) -> bool {
        self.paths.settings_file().exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        let storage = Storage::new(paths).unwrap();

        assert!(temp_dir.path().join("data").exists());
        assert!(temp_dir.path().join("backups").exists());
        assert!(!storage.is_initialized());
    }
}
