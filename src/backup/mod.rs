//! Backup system for EnvelopeCLI
//!
//! Provides automatic rolling backups with configurable retention policies
//! and restore functionality.
//!
//! # Architecture
//!
//! The backup system consists of two main components:
//!
//! - `BackupManager`: Creates and manages backups with retention policies
//! - `RestoreManager`: Validates and restores backups
//!
//! # Backup Format
//!
//! Backups are stored as JSON files with the following structure:
//! - `schema_version`: Version for migration support
//! - `created_at`: Timestamp when backup was created
//! - `accounts`: Account data
//! - `transactions`: Transaction data
//! - `budget`: Categories, groups, and allocations
//! - `payees`: Payee data
//!
//! # Retention Policy
//!
//! By default, the system keeps:
//! - 30 daily backups
//! - 12 monthly backups (first backup of each month)
//!
//! # Example
//!
//! ```rust,ignore
//! use envelope::backup::{BackupManager, RestoreManager};
//! use envelope::config::{paths::EnvelopePaths, settings::BackupRetention};
//!
//! // Create a backup
//! let paths = EnvelopePaths::new()?;
//! let retention = BackupRetention::default();
//! let backup_manager = BackupManager::new(paths.clone(), retention);
//!
//! let backup_path = backup_manager.create_backup()?;
//! backup_manager.enforce_retention()?;
//!
//! // Later, restore from backup
//! let restore_manager = RestoreManager::new(paths);
//! let result = restore_manager.restore_from_file(&backup_path)?;
//! println!("{}", result.summary());
//! ```

mod manager;
mod restore;

pub use manager::{BackupArchive, BackupInfo, BackupManager};
pub use restore::{RestoreManager, RestoreResult, ValidationResult};
