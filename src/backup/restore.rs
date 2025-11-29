//! Backup restoration for EnvelopeCLI
//!
//! Handles restoring data from backup archives.
//! Supports both internal backup format and export format files.

use std::fs;
use std::path::Path;

use crate::config::paths::EnvelopePaths;
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::export::FullExport;

use super::manager::BackupArchive;

/// File format type detected during parsing
#[derive(Debug)]
pub enum BackupFileFormat {
    /// Internal backup format (schema_version as u32)
    Backup(BackupArchive),
    /// Export format (schema_version as semver string)
    Export(FullExport),
}

/// Parse backup file contents, auto-detecting format
///
/// Tries to parse as export format first (which has string schema_version),
/// then falls back to internal backup format (which has u32 schema_version).
fn parse_backup_contents(path: &Path, contents: &str) -> EnvelopeResult<BackupFileFormat> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "yaml" | "yml" => {
            // Try export format first (has string schema_version)
            if let Ok(export) = serde_yaml::from_str::<FullExport>(contents) {
                return Ok(BackupFileFormat::Export(export));
            }
            // Fall back to backup format
            serde_yaml::from_str::<BackupArchive>(contents)
                .map(BackupFileFormat::Backup)
                .map_err(|e| {
                    EnvelopeError::Json(format!("Failed to parse YAML backup file: {}", e))
                })
        }
        _ => {
            // Try export format first (has string schema_version)
            if let Ok(export) = serde_json::from_str::<FullExport>(contents) {
                return Ok(BackupFileFormat::Export(export));
            }
            // Fall back to backup format
            serde_json::from_str::<BackupArchive>(contents)
                .map(BackupFileFormat::Backup)
                .map_err(|e| EnvelopeError::Json(format!("Failed to parse backup file: {}", e)))
        }
    }
}

/// Handles restoring from backups
pub struct RestoreManager {
    paths: EnvelopePaths,
}

impl RestoreManager {
    /// Create a new RestoreManager
    pub fn new(paths: EnvelopePaths) -> Self {
        Self { paths }
    }

    /// Restore data from a backup file
    ///
    /// This will overwrite all current data with the backup contents.
    /// It's recommended to create a backup before restoring.
    /// Supports both internal backup format and export format files.
    /// Supports both JSON and YAML formats (detected by file extension).
    pub fn restore_from_file(&self, backup_path: &Path) -> EnvelopeResult<RestoreResult> {
        // Read and parse the backup
        let contents = fs::read_to_string(backup_path)
            .map_err(|e| EnvelopeError::Io(format!("Failed to read backup file: {}", e)))?;

        let parsed = parse_backup_contents(backup_path, &contents)?;

        match parsed {
            BackupFileFormat::Backup(archive) => self.restore_from_archive(&archive),
            BackupFileFormat::Export(export) => self.restore_from_export(&export),
        }
    }

    /// Restore data from a parsed backup archive (internal format)
    pub fn restore_from_archive(&self, archive: &BackupArchive) -> EnvelopeResult<RestoreResult> {
        // Ensure directories exist
        self.paths.ensure_directories()?;

        let mut result = RestoreResult::default();

        // Restore accounts
        if !archive.accounts.is_null() {
            let json = serde_json::to_string_pretty(&archive.accounts)
                .map_err(|e| EnvelopeError::Json(format!("Failed to serialize accounts: {}", e)))?;
            fs::write(self.paths.accounts_file(), json)
                .map_err(|e| EnvelopeError::Io(format!("Failed to restore accounts: {}", e)))?;
            result.accounts_restored = true;
        }

        // Restore transactions
        if !archive.transactions.is_null() {
            let json = serde_json::to_string_pretty(&archive.transactions).map_err(|e| {
                EnvelopeError::Json(format!("Failed to serialize transactions: {}", e))
            })?;
            fs::write(self.paths.transactions_file(), json)
                .map_err(|e| EnvelopeError::Io(format!("Failed to restore transactions: {}", e)))?;
            result.transactions_restored = true;
        }

        // Restore budget (categories, groups, allocations)
        if !archive.budget.is_null() {
            let json = serde_json::to_string_pretty(&archive.budget)
                .map_err(|e| EnvelopeError::Json(format!("Failed to serialize budget: {}", e)))?;
            fs::write(self.paths.budget_file(), json)
                .map_err(|e| EnvelopeError::Io(format!("Failed to restore budget: {}", e)))?;
            result.budget_restored = true;
        }

        // Restore payees
        if !archive.payees.is_null() {
            let json = serde_json::to_string_pretty(&archive.payees)
                .map_err(|e| EnvelopeError::Json(format!("Failed to serialize payees: {}", e)))?;
            fs::write(self.paths.payees_file(), json)
                .map_err(|e| EnvelopeError::Io(format!("Failed to restore payees: {}", e)))?;
            result.payees_restored = true;
        }

        result.schema_version = archive.schema_version;
        result.backup_date = archive.created_at;
        result.is_export_format = false;

        Ok(result)
    }

    /// Restore data from an export file
    fn restore_from_export(&self, export: &FullExport) -> EnvelopeResult<RestoreResult> {
        // Ensure directories exist
        self.paths.ensure_directories()?;

        // Create a storage instance to use the proper upsert methods
        let storage = crate::storage::Storage::new(self.paths.clone())?;

        // Use the export restore function
        let export_result = crate::export::restore_from_export(&storage, export)?;

        // Convert to RestoreResult
        Ok(RestoreResult {
            schema_version: 1, // Export files use semver, convert to internal version
            backup_date: export_result.exported_at,
            accounts_restored: export_result.accounts_restored > 0,
            transactions_restored: export_result.transactions_restored > 0,
            budget_restored: export_result.categories_restored > 0
                || export_result.category_groups_restored > 0
                || export_result.allocations_restored > 0,
            payees_restored: export_result.payees_restored > 0,
            is_export_format: true,
            export_schema_version: Some(export_result.schema_version),
            export_counts: Some(ExportRestoreCounts {
                accounts: export_result.accounts_restored,
                category_groups: export_result.category_groups_restored,
                categories: export_result.categories_restored,
                transactions: export_result.transactions_restored,
                allocations: export_result.allocations_restored,
                payees: export_result.payees_restored,
            }),
        })
    }

    /// Validate a backup file without restoring it
    /// Supports both internal backup format and export format files.
    /// Supports both JSON and YAML formats (detected by file extension).
    pub fn validate_backup(&self, backup_path: &Path) -> EnvelopeResult<ValidationResult> {
        let contents = fs::read_to_string(backup_path)
            .map_err(|e| EnvelopeError::Io(format!("Failed to read backup file: {}", e)))?;

        let parsed = parse_backup_contents(backup_path, &contents)?;

        match parsed {
            BackupFileFormat::Backup(archive) => Ok(ValidationResult {
                is_valid: true,
                schema_version: archive.schema_version,
                backup_date: archive.created_at,
                has_accounts: !archive.accounts.is_null() && archive.accounts.is_object(),
                has_transactions: !archive.transactions.is_null()
                    && archive.transactions.is_object(),
                has_budget: !archive.budget.is_null() && archive.budget.is_object(),
                has_payees: !archive.payees.is_null() && archive.payees.is_object(),
                is_export_format: false,
                export_schema_version: None,
            }),
            BackupFileFormat::Export(export) => Ok(ValidationResult {
                is_valid: true,
                schema_version: 1, // Export files use semver, report as v1
                backup_date: export.exported_at,
                has_accounts: !export.accounts.is_empty(),
                has_transactions: !export.transactions.is_empty(),
                has_budget: !export.categories.is_empty() || !export.category_groups.is_empty(),
                has_payees: !export.payees.is_empty(),
                is_export_format: true,
                export_schema_version: Some(export.schema_version),
            }),
        }
    }
}

/// Detailed counts for export format restores
#[derive(Debug, Default, Clone)]
pub struct ExportRestoreCounts {
    /// Number of accounts restored
    pub accounts: usize,
    /// Number of category groups restored
    pub category_groups: usize,
    /// Number of categories restored
    pub categories: usize,
    /// Number of transactions restored
    pub transactions: usize,
    /// Number of allocations restored
    pub allocations: usize,
    /// Number of payees restored
    pub payees: usize,
}

/// Result of a restore operation
#[derive(Debug, Default)]
pub struct RestoreResult {
    /// Schema version of the restored backup
    pub schema_version: u32,
    /// Date the backup was created
    pub backup_date: chrono::DateTime<chrono::Utc>,
    /// Whether accounts were restored
    pub accounts_restored: bool,
    /// Whether transactions were restored
    pub transactions_restored: bool,
    /// Whether budget data was restored
    pub budget_restored: bool,
    /// Whether payees were restored
    pub payees_restored: bool,
    /// Whether this was restored from an export format file
    pub is_export_format: bool,
    /// Schema version string for export format files
    pub export_schema_version: Option<String>,
    /// Detailed counts for export format restores
    pub export_counts: Option<ExportRestoreCounts>,
}

impl RestoreResult {
    /// Check if all data was restored
    pub fn all_restored(&self) -> bool {
        self.accounts_restored
            && self.transactions_restored
            && self.budget_restored
            && self.payees_restored
    }

    /// Get a summary of what was restored
    pub fn summary(&self) -> String {
        if let Some(counts) = &self.export_counts {
            format!(
                "Restored: {} accounts, {} groups, {} categories, {} transactions, {} allocations, {} payees",
                counts.accounts,
                counts.category_groups,
                counts.categories,
                counts.transactions,
                counts.allocations,
                counts.payees
            )
        } else {
            let mut parts = Vec::new();
            if self.accounts_restored {
                parts.push("accounts");
            }
            if self.transactions_restored {
                parts.push("transactions");
            }
            if self.budget_restored {
                parts.push("budget");
            }
            if self.payees_restored {
                parts.push("payees");
            }
            format!("Restored: {}", parts.join(", "))
        }
    }
}

/// Result of validating a backup
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether the backup file is valid
    pub is_valid: bool,
    /// Schema version of the backup
    pub schema_version: u32,
    /// Date the backup was created
    pub backup_date: chrono::DateTime<chrono::Utc>,
    /// Whether backup contains accounts data
    pub has_accounts: bool,
    /// Whether backup contains transactions data
    pub has_transactions: bool,
    /// Whether backup contains budget data
    pub has_budget: bool,
    /// Whether backup contains payees data
    pub has_payees: bool,
    /// Whether this is an export format file
    pub is_export_format: bool,
    /// Schema version string for export format files
    pub export_schema_version: Option<String>,
}

impl ValidationResult {
    /// Check if all expected data is present
    pub fn is_complete(&self) -> bool {
        self.has_accounts && self.has_transactions && self.has_budget && self.has_payees
    }

    /// Get a summary of what data is present
    pub fn summary(&self) -> String {
        let mut present = Vec::new();
        let mut missing = Vec::new();

        if self.has_accounts {
            present.push("accounts");
        } else {
            missing.push("accounts");
        }
        if self.has_transactions {
            present.push("transactions");
        } else {
            missing.push("transactions");
        }
        if self.has_budget {
            present.push("budget");
        } else {
            missing.push("budget");
        }
        if self.has_payees {
            present.push("payees");
        } else {
            missing.push("payees");
        }

        let version_str = if let Some(ref export_ver) = self.export_schema_version {
            format!("v{}", export_ver)
        } else {
            format!("v{}", self.schema_version)
        };

        let format_str = if self.is_export_format {
            "export"
        } else {
            "backup"
        };

        if missing.is_empty() {
            format!("Complete {} ({})", format_str, version_str)
        } else {
            format!(
                "Partial {} ({}): has {}, missing {}",
                format_str,
                version_str,
                present.join(", "),
                missing.join(", ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::manager::BackupManager;
    use crate::config::settings::BackupRetention;
    use tempfile::TempDir;

    fn create_test_env() -> (RestoreManager, BackupManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        paths.ensure_directories().unwrap();

        let retention = BackupRetention::default();
        let backup_manager = BackupManager::new(paths.clone(), retention);
        let restore_manager = RestoreManager::new(paths);

        (restore_manager, backup_manager, temp_dir)
    }

    #[test]
    fn test_restore_from_backup() {
        let (restore_manager, backup_manager, _temp) = create_test_env();

        // Create a backup
        let backup_path = backup_manager.create_backup().unwrap();

        // Restore from it
        let result = restore_manager.restore_from_file(&backup_path).unwrap();

        assert!(result.accounts_restored);
        assert!(result.transactions_restored);
        assert!(result.budget_restored);
        assert!(result.payees_restored);
    }

    #[test]
    fn test_validate_backup() {
        let (restore_manager, backup_manager, _temp) = create_test_env();

        // Create a backup
        let backup_path = backup_manager.create_backup().unwrap();

        // Validate it
        let result = restore_manager.validate_backup(&backup_path).unwrap();

        assert!(result.is_valid);
        assert_eq!(result.schema_version, 1);
    }

    #[test]
    fn test_restore_result_summary() {
        let result = RestoreResult {
            schema_version: 1,
            backup_date: chrono::Utc::now(),
            accounts_restored: true,
            transactions_restored: true,
            budget_restored: false,
            payees_restored: true,
            is_export_format: false,
            export_schema_version: None,
            export_counts: None,
        };

        assert!(!result.all_restored());
        assert!(result.summary().contains("accounts"));
        assert!(result.summary().contains("transactions"));
        assert!(!result.summary().contains("budget"));
    }

    #[test]
    fn test_validation_result_summary() {
        let result = ValidationResult {
            is_valid: true,
            schema_version: 1,
            backup_date: chrono::Utc::now(),
            has_accounts: true,
            has_transactions: true,
            has_budget: true,
            has_payees: true,
            is_export_format: false,
            export_schema_version: None,
        };

        assert!(result.is_complete());
        assert!(result.summary().contains("Complete backup"));
    }

    #[test]
    fn test_restore_creates_files() {
        let (restore_manager, backup_manager, temp) = create_test_env();

        // Create backup with some data
        let backup_path = backup_manager.create_backup().unwrap();

        // Delete the data files
        let data_dir = temp.path().join("data");
        if data_dir.exists() {
            fs::remove_dir_all(&data_dir).unwrap();
        }

        // Restore should recreate them
        restore_manager.restore_from_file(&backup_path).unwrap();

        // Check files exist
        assert!(restore_manager.paths.accounts_file().exists());
        assert!(restore_manager.paths.transactions_file().exists());
        assert!(restore_manager.paths.budget_file().exists());
        assert!(restore_manager.paths.payees_file().exists());
    }
}
