//! Backup manager for EnvelopeCLI
//!
//! Handles automatic rolling backups with configurable retention policies.
//! Backups are stored as dated JSON archives.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::config::paths::EnvelopePaths;
use crate::config::settings::BackupRetention;
use crate::error::{EnvelopeError, EnvelopeResult};

/// Metadata about a backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Backup filename
    pub filename: String,
    /// Full path to backup
    pub path: PathBuf,
    /// When the backup was created
    pub created_at: DateTime<Utc>,
    /// Size in bytes
    pub size_bytes: u64,
    /// Whether this is a monthly backup (kept longer)
    pub is_monthly: bool,
}

/// Backup archive format
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupArchive {
    /// Schema version for migration support
    pub schema_version: u32,
    /// When the backup was created
    pub created_at: DateTime<Utc>,
    /// Accounts data
    pub accounts: serde_json::Value,
    /// Transactions data
    pub transactions: serde_json::Value,
    /// Budget data (categories, groups, allocations)
    pub budget: serde_json::Value,
    /// Payees data
    pub payees: serde_json::Value,
}

/// Manages backup creation and retention
pub struct BackupManager {
    /// Path to backup directory
    backup_dir: PathBuf,
    /// Paths to data files
    paths: EnvelopePaths,
    /// Retention policy
    retention: BackupRetention,
}

impl BackupManager {
    /// Create a new BackupManager
    pub fn new(paths: EnvelopePaths, retention: BackupRetention) -> Self {
        let backup_dir = paths.backup_dir();
        Self {
            backup_dir,
            paths,
            retention,
        }
    }

    /// Create a backup of all data
    ///
    /// Returns the path to the created backup file.
    pub fn create_backup(&self) -> EnvelopeResult<PathBuf> {
        // Ensure backup directory exists
        fs::create_dir_all(&self.backup_dir).map_err(|e| {
            EnvelopeError::Io(format!("Failed to create backup directory: {}", e))
        })?;

        let now = Utc::now();
        let filename = format!("backup-{}-{:03}.json", now.format("%Y%m%d-%H%M%S"), now.timestamp_subsec_millis());
        let backup_path = self.backup_dir.join(&filename);

        // Read all data files
        let archive = self.create_archive(now)?;

        // Write backup file
        let json = serde_json::to_string_pretty(&archive).map_err(|e| {
            EnvelopeError::Json(format!("Failed to serialize backup: {}", e))
        })?;

        fs::write(&backup_path, json).map_err(|e| {
            EnvelopeError::Io(format!("Failed to write backup file: {}", e))
        })?;

        Ok(backup_path)
    }

    /// Create a backup archive from current data
    fn create_archive(&self, timestamp: DateTime<Utc>) -> EnvelopeResult<BackupArchive> {
        Ok(BackupArchive {
            schema_version: 1,
            created_at: timestamp,
            accounts: read_json_value(&self.paths.accounts_file())?,
            transactions: read_json_value(&self.paths.transactions_file())?,
            budget: read_json_value(&self.paths.budget_file())?,
            payees: read_json_value(&self.paths.payees_file())?,
        })
    }

    /// List all available backups
    pub fn list_backups(&self) -> EnvelopeResult<Vec<BackupInfo>> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        for entry in fs::read_dir(&self.backup_dir).map_err(|e| {
            EnvelopeError::Io(format!("Failed to read backup directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                EnvelopeError::Io(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(info) = self.parse_backup_info(&path) {
                    backups.push(info);
                }
            }
        }

        // Sort by date, newest first
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    /// Parse backup info from a backup file
    fn parse_backup_info(&self, path: &Path) -> Option<BackupInfo> {
        let filename = path.file_name()?.to_string_lossy().to_string();

        // Parse date from filename: backup-YYYYMMDD-HHMMSS.json
        if !filename.starts_with("backup-") {
            return None;
        }

        let date_part = filename.strip_prefix("backup-")?.strip_suffix(".json")?;
        let created_at = parse_backup_timestamp(date_part)?;

        let metadata = fs::metadata(path).ok()?;
        let size_bytes = metadata.len();

        // A backup is "monthly" if it's the first backup of the month
        let is_monthly = self.is_first_of_month(&created_at);

        Some(BackupInfo {
            filename,
            path: path.to_path_buf(),
            created_at,
            size_bytes,
            is_monthly,
        })
    }

    /// Check if this backup is the first of its month
    fn is_first_of_month(&self, timestamp: &DateTime<Utc>) -> bool {
        timestamp.day() == 1
    }

    /// Enforce retention policy by deleting old backups
    pub fn enforce_retention(&self) -> EnvelopeResult<Vec<PathBuf>> {
        let backups = self.list_backups()?;
        let mut deleted = Vec::new();

        // Separate daily and monthly backups
        let (monthly, daily): (Vec<_>, Vec<_>) = backups
            .into_iter()
            .partition(|b| b.is_monthly);

        // Keep only the configured number of daily backups
        for backup in daily.into_iter().skip(self.retention.daily_count as usize) {
            fs::remove_file(&backup.path).map_err(|e| {
                EnvelopeError::Io(format!("Failed to delete old backup: {}", e))
            })?;
            deleted.push(backup.path);
        }

        // Keep only the configured number of monthly backups
        for backup in monthly.into_iter().skip(self.retention.monthly_count as usize) {
            fs::remove_file(&backup.path).map_err(|e| {
                EnvelopeError::Io(format!("Failed to delete old monthly backup: {}", e))
            })?;
            deleted.push(backup.path);
        }

        Ok(deleted)
    }

    /// Create a backup and then enforce retention policy
    pub fn create_backup_with_retention(&self) -> EnvelopeResult<(PathBuf, Vec<PathBuf>)> {
        let backup_path = self.create_backup()?;
        let deleted = self.enforce_retention()?;
        Ok((backup_path, deleted))
    }

    /// Get backup directory path
    pub fn backup_dir(&self) -> &PathBuf {
        &self.backup_dir
    }

    /// Get a specific backup by filename
    pub fn get_backup(&self, filename: &str) -> EnvelopeResult<Option<BackupInfo>> {
        let path = self.backup_dir.join(filename);
        if path.exists() {
            Ok(self.parse_backup_info(&path))
        } else {
            Ok(None)
        }
    }

    /// Get the most recent backup
    pub fn get_latest_backup(&self) -> EnvelopeResult<Option<BackupInfo>> {
        let backups = self.list_backups()?;
        Ok(backups.into_iter().next())
    }
}

/// Read a JSON file as a generic Value, returning empty object if file doesn't exist
fn read_json_value(path: &Path) -> EnvelopeResult<serde_json::Value> {
    if !path.exists() {
        return Ok(serde_json::Value::Object(serde_json::Map::new()));
    }

    let contents = fs::read_to_string(path).map_err(|e| {
        EnvelopeError::Io(format!("Failed to read file for backup: {}", e))
    })?;

    serde_json::from_str(&contents).map_err(|e| {
        EnvelopeError::Json(format!("Failed to parse JSON for backup: {}", e))
    })
}

/// Parse a backup timestamp from the filename date part
fn parse_backup_timestamp(date_str: &str) -> Option<DateTime<Utc>> {
    // Expected format: YYYYMMDD-HHMMSS or YYYYMMDD-HHMMSS-mmm (with milliseconds)
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }

    let date_part = parts[0];
    let time_part = parts[1];
    let millis: u32 = if parts.len() == 3 {
        parts[2].parse().unwrap_or(0)
    } else {
        0
    };

    if date_part.len() != 8 || time_part.len() != 6 {
        return None;
    }

    let year: i32 = date_part[0..4].parse().ok()?;
    let month: u32 = date_part[4..6].parse().ok()?;
    let day: u32 = date_part[6..8].parse().ok()?;
    let hour: u32 = time_part[0..2].parse().ok()?;
    let minute: u32 = time_part[2..4].parse().ok()?;
    let second: u32 = time_part[4..6].parse().ok()?;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let time = chrono::NaiveTime::from_hms_milli_opt(hour, minute, second, millis)?;
    let datetime = chrono::NaiveDateTime::new(date, time);

    Some(DateTime::from_naive_utc_and_offset(datetime, Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manager() -> (BackupManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        paths.ensure_directories().unwrap();

        let retention = BackupRetention {
            daily_count: 3,
            monthly_count: 2,
        };

        let manager = BackupManager::new(paths, retention);
        (manager, temp_dir)
    }

    #[test]
    fn test_create_backup() {
        let (manager, _temp) = create_test_manager();

        let backup_path = manager.create_backup().unwrap();
        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains("backup-"));
    }

    #[test]
    fn test_list_backups() {
        let (manager, _temp) = create_test_manager();

        // Create a few backups
        manager.create_backup().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        manager.create_backup().unwrap();

        let backups = manager.list_backups().unwrap();
        assert_eq!(backups.len(), 2);

        // Should be sorted newest first
        assert!(backups[0].created_at >= backups[1].created_at);
    }

    #[test]
    fn test_retention_policy() {
        let (manager, _temp) = create_test_manager();

        // Create more backups than retention allows
        for _ in 0..5 {
            manager.create_backup().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let deleted = manager.enforce_retention().unwrap();
        assert_eq!(deleted.len(), 2); // 5 - 3 = 2 deleted

        let remaining = manager.list_backups().unwrap();
        assert_eq!(remaining.len(), 3);
    }

    #[test]
    fn test_get_latest_backup() {
        let (manager, _temp) = create_test_manager();

        // No backups yet
        assert!(manager.get_latest_backup().unwrap().is_none());

        // Create backup
        let path = manager.create_backup().unwrap();

        let latest = manager.get_latest_backup().unwrap().unwrap();
        assert_eq!(latest.path, path);
    }

    #[test]
    fn test_parse_backup_timestamp() {
        // Test old format without milliseconds
        let timestamp = parse_backup_timestamp("20251127-143022").unwrap();
        assert_eq!(timestamp.year(), 2025);
        assert_eq!(timestamp.month(), 11);
        assert_eq!(timestamp.day(), 27);

        // Test new format with milliseconds
        let timestamp = parse_backup_timestamp("20251127-143022-456").unwrap();
        assert_eq!(timestamp.year(), 2025);
        assert_eq!(timestamp.month(), 11);
        assert_eq!(timestamp.day(), 27);
    }

    #[test]
    fn test_backup_archive_structure() {
        let (manager, _temp) = create_test_manager();

        let backup_path = manager.create_backup().unwrap();

        // Read and parse the backup
        let contents = fs::read_to_string(&backup_path).unwrap();
        let archive: BackupArchive = serde_json::from_str(&contents).unwrap();

        assert_eq!(archive.schema_version, 1);
        assert!(archive.accounts.is_object());
        assert!(archive.transactions.is_object());
    }

    #[test]
    fn test_empty_backup_dir() {
        let (manager, _temp) = create_test_manager();

        let backups = manager.list_backups().unwrap();
        assert!(backups.is_empty());
    }

    #[test]
    fn test_create_backup_with_retention() {
        let (manager, _temp) = create_test_manager();

        // Create initial backups
        for _ in 0..5 {
            manager.create_backup().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // This should create one more and delete old ones
        let (new_backup, deleted) = manager.create_backup_with_retention().unwrap();

        assert!(new_backup.exists());
        assert!(!deleted.is_empty());
    }
}
