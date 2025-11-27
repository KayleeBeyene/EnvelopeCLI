//! Storage layer for EnvelopeCLI
//!
//! Provides JSON file storage with atomic writes, file locking, and
//! automatic directory creation. Includes audit logging for all
//! create, update, and delete operations.

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

use std::path::PathBuf;

use crate::audit::{AuditEntry, AuditLogger, EntityType};
use crate::backup::{BackupManager, RestoreManager, RestoreResult};
use crate::config::paths::EnvelopePaths;
use crate::config::settings::BackupRetention;
use crate::error::{EnvelopeError, EnvelopeResult};

/// Main storage coordinator that provides access to all repositories
/// and handles audit logging for all operations.
pub struct Storage {
    paths: EnvelopePaths,
    pub accounts: AccountRepository,
    pub transactions: TransactionRepository,
    pub categories: CategoryRepository,
    pub budget: BudgetRepository,
    pub payees: PayeeRepository,
    audit: AuditLogger,
}

impl Storage {
    /// Create a new Storage instance
    pub fn new(paths: EnvelopePaths) -> Result<Self, EnvelopeError> {
        // Ensure directories exist
        paths.ensure_directories()?;

        let audit = AuditLogger::new(paths.audit_log());

        Ok(Self {
            accounts: AccountRepository::new(paths.accounts_file()),
            transactions: TransactionRepository::new(paths.transactions_file()),
            categories: CategoryRepository::new(paths.budget_file()),
            budget: BudgetRepository::new(paths.allocations_file()),
            payees: PayeeRepository::new(paths.payees_file()),
            audit,
            paths,
        })
    }

    /// Get the paths configuration
    pub fn paths(&self) -> &EnvelopePaths {
        &self.paths
    }

    /// Get a reference to the audit logger
    pub fn audit(&self) -> &AuditLogger {
        &self.audit
    }

    /// Log an audit entry
    pub fn log_audit(&self, entry: &AuditEntry) -> EnvelopeResult<()> {
        self.audit.log(entry)
    }

    /// Log a create operation
    pub fn log_create<T: serde::Serialize>(
        &self,
        entity_type: EntityType,
        entity_id: impl Into<String>,
        entity_name: Option<String>,
        entity: &T,
    ) -> EnvelopeResult<()> {
        let entry = AuditEntry::create(entity_type, entity_id, entity_name, entity);
        self.audit.log(&entry)
    }

    /// Log an update operation
    pub fn log_update<T: serde::Serialize>(
        &self,
        entity_type: EntityType,
        entity_id: impl Into<String>,
        entity_name: Option<String>,
        before: &T,
        after: &T,
        diff_summary: Option<String>,
    ) -> EnvelopeResult<()> {
        let entry = AuditEntry::update(entity_type, entity_id, entity_name, before, after, diff_summary);
        self.audit.log(&entry)
    }

    /// Log a delete operation
    pub fn log_delete<T: serde::Serialize>(
        &self,
        entity_type: EntityType,
        entity_id: impl Into<String>,
        entity_name: Option<String>,
        entity: &T,
    ) -> EnvelopeResult<()> {
        let entry = AuditEntry::delete(entity_type, entity_id, entity_name, entity);
        self.audit.log(&entry)
    }

    /// Read recent audit entries
    pub fn read_audit_log(&self, count: usize) -> EnvelopeResult<Vec<AuditEntry>> {
        self.audit.read_recent(count)
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

    /// Create a backup of all data
    ///
    /// Creates a backup using the default retention policy.
    /// Returns the path to the created backup file.
    pub fn create_backup(&self) -> EnvelopeResult<PathBuf> {
        let retention = BackupRetention::default();
        let manager = BackupManager::new(self.paths.clone(), retention);
        manager.create_backup()
    }

    /// Create a backup with a custom retention policy
    pub fn create_backup_with_retention(
        &self,
        retention: BackupRetention,
    ) -> EnvelopeResult<(PathBuf, Vec<PathBuf>)> {
        let manager = BackupManager::new(self.paths.clone(), retention);
        manager.create_backup_with_retention()
    }

    /// Restore data from a backup file
    ///
    /// WARNING: This will overwrite all current data.
    /// It's recommended to create a backup before restoring.
    pub fn restore_from_backup(&mut self, backup_path: &PathBuf) -> EnvelopeResult<RestoreResult> {
        let restore_manager = RestoreManager::new(self.paths.clone());
        let result = restore_manager.restore_from_file(backup_path)?;

        // Reload all repositories after restore
        self.load_all()?;

        Ok(result)
    }

    /// Get the backup manager for advanced backup operations
    pub fn backup_manager(&self, retention: BackupRetention) -> BackupManager {
        BackupManager::new(self.paths.clone(), retention)
    }

    /// Create a backup before a destructive operation if needed
    ///
    /// This creates a backup only if:
    /// - No backup exists yet, OR
    /// - The most recent backup is older than 60 seconds
    ///
    /// This prevents creating too many backups when multiple destructive
    /// operations happen in quick succession.
    ///
    /// Returns Ok(Some(path)) if a backup was created, Ok(None) if skipped.
    pub fn backup_before_destructive(&self) -> EnvelopeResult<Option<PathBuf>> {
        let retention = BackupRetention::default();
        let manager = BackupManager::new(self.paths.clone(), retention);

        // Check if we need to create a backup
        if let Some(latest) = manager.get_latest_backup()? {
            let age = chrono::Utc::now()
                .signed_duration_since(latest.created_at);

            // Skip if last backup was less than 60 seconds ago
            if age.num_seconds() < 60 {
                return Ok(None);
            }
        }

        // Create backup
        let path = manager.create_backup()?;
        Ok(Some(path))
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
