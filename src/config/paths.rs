//! Path management for EnvelopeCLI
//!
//! Provides XDG-compliant path resolution for configuration, data, and backups.
//! Default location: ~/.envelope/

use std::path::PathBuf;

use directories::ProjectDirs;

use crate::error::EnvelopeError;

/// Manages all paths used by EnvelopeCLI
#[derive(Debug, Clone)]
pub struct EnvelopePaths {
    /// Base directory for all EnvelopeCLI data (~/.envelope/)
    base_dir: PathBuf,
}

impl EnvelopePaths {
    /// Create a new EnvelopePaths instance using XDG directories
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined.
    pub fn new() -> Result<Self, EnvelopeError> {
        let base_dir = if let Some(proj_dirs) = ProjectDirs::from("", "", "envelope") {
            proj_dirs.data_dir().to_path_buf()
        } else {
            // Fallback to ~/.envelope if XDG fails
            dirs_fallback()?
        };

        Ok(Self { base_dir })
    }

    /// Create EnvelopePaths with a custom base directory (useful for testing)
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Get the base directory (~/.envelope/ or equivalent)
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// Get the config directory (same as base for simplicity)
    pub fn config_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }

    /// Get the data directory (~/.envelope/data/)
    pub fn data_dir(&self) -> PathBuf {
        self.base_dir.join("data")
    }

    /// Get the backup directory (~/.envelope/backups/)
    pub fn backup_dir(&self) -> PathBuf {
        self.base_dir.join("backups")
    }

    /// Get the path to the settings file
    pub fn settings_file(&self) -> PathBuf {
        self.base_dir.join("config.json")
    }

    /// Get the path to the audit log
    pub fn audit_log(&self) -> PathBuf {
        self.base_dir.join("audit.log")
    }

    /// Get the path to accounts.json
    pub fn accounts_file(&self) -> PathBuf {
        self.data_dir().join("accounts.json")
    }

    /// Get the path to transactions.json
    pub fn transactions_file(&self) -> PathBuf {
        self.data_dir().join("transactions.json")
    }

    /// Get the path to budget.json (categories and groups)
    pub fn budget_file(&self) -> PathBuf {
        self.data_dir().join("budget.json")
    }

    /// Get the path to allocations.json (budget allocations per period)
    pub fn allocations_file(&self) -> PathBuf {
        self.data_dir().join("allocations.json")
    }

    /// Get the path to payees.json
    pub fn payees_file(&self) -> PathBuf {
        self.data_dir().join("payees.json")
    }

    /// Ensure all required directories exist
    ///
    /// Creates:
    /// - Base directory (~/.envelope/)
    /// - Data directory (~/.envelope/data/)
    /// - Backup directory (~/.envelope/backups/)
    pub fn ensure_directories(&self) -> Result<(), EnvelopeError> {
        std::fs::create_dir_all(&self.base_dir)
            .map_err(|e| EnvelopeError::Io(format!("Failed to create base directory: {}", e)))?;

        std::fs::create_dir_all(self.data_dir())
            .map_err(|e| EnvelopeError::Io(format!("Failed to create data directory: {}", e)))?;

        std::fs::create_dir_all(self.backup_dir())
            .map_err(|e| EnvelopeError::Io(format!("Failed to create backup directory: {}", e)))?;

        Ok(())
    }

    /// Check if EnvelopeCLI has been initialized (config file exists)
    pub fn is_initialized(&self) -> bool {
        self.settings_file().exists()
    }
}

/// Fallback path resolution when XDG directories aren't available
fn dirs_fallback() -> Result<PathBuf, EnvelopeError> {
    let home = std::env::var("HOME")
        .map_err(|_| EnvelopeError::Config("Could not determine home directory".into()))?;
    Ok(PathBuf::from(home).join(".envelope"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_custom_base_dir() {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());

        assert_eq!(paths.base_dir(), temp_dir.path());
        assert_eq!(paths.data_dir(), temp_dir.path().join("data"));
        assert_eq!(paths.backup_dir(), temp_dir.path().join("backups"));
    }

    #[test]
    fn test_ensure_directories() {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());

        paths.ensure_directories().unwrap();

        assert!(paths.data_dir().exists());
        assert!(paths.backup_dir().exists());
    }

    #[test]
    fn test_file_paths() {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());

        assert_eq!(paths.settings_file(), temp_dir.path().join("config.json"));
        assert_eq!(
            paths.accounts_file(),
            temp_dir.path().join("data").join("accounts.json")
        );
    }
}
