//! Backup CLI commands
//!
//! Implements CLI commands for backup management.

use clap::Subcommand;
use std::path::PathBuf;

use crate::backup::{BackupManager, RestoreManager};
use crate::config::paths::EnvelopePaths;
use crate::config::settings::Settings;
use crate::error::EnvelopeResult;

/// Backup subcommands
#[derive(Subcommand)]
pub enum BackupCommands {
    /// Create a new backup
    Create,

    /// List all available backups
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Restore from a backup
    Restore {
        /// Backup filename or path (use 'latest' for most recent)
        backup: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Show information about a specific backup
    Info {
        /// Backup filename or path
        backup: String,
    },

    /// Delete old backups according to retention policy
    Prune {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

/// Handle a backup command
pub fn handle_backup_command(
    paths: &EnvelopePaths,
    settings: &Settings,
    cmd: BackupCommands,
) -> EnvelopeResult<()> {
    let retention = settings.backup_retention.clone();
    let manager = BackupManager::new(paths.clone(), retention);

    match cmd {
        BackupCommands::Create => {
            println!("Creating backup...");
            let backup_path = manager.create_backup()?;
            let filename = backup_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| backup_path.display().to_string());
            println!("Backup created: {}", filename);
            println!("Location: {}", backup_path.display());
        }

        BackupCommands::List { verbose } => {
            let backups = manager.list_backups()?;

            if backups.is_empty() {
                println!("No backups found.");
                println!("Create one with: envelope backup create");
                return Ok(());
            }

            println!("Available Backups");
            println!("=================");
            println!();

            for (i, backup) in backups.iter().enumerate() {
                let age = chrono::Utc::now().signed_duration_since(backup.created_at);
                let age_str = format_duration(age);

                let monthly_marker = if backup.is_monthly { " [monthly]" } else { "" };

                if verbose {
                    println!(
                        "{}. {}{}\n   Created: {}\n   Size: {}\n   Age: {}\n",
                        i + 1,
                        backup.filename,
                        monthly_marker,
                        backup.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
                        format_size(backup.size_bytes),
                        age_str,
                    );
                } else {
                    println!(
                        "  {}. {} ({} ago, {}){}",
                        i + 1,
                        backup.filename,
                        age_str,
                        format_size(backup.size_bytes),
                        monthly_marker,
                    );
                }
            }

            println!();
            println!("Total: {} backup(s)", backups.len());
        }

        BackupCommands::Restore { backup, force } => {
            let backup_path = resolve_backup_path(&manager, paths, &backup)?;

            // Validate the backup first
            let restore_manager = RestoreManager::new(paths.clone());
            let validation = restore_manager.validate_backup(&backup_path)?;

            println!("Backup Information");
            println!("==================");
            println!("File: {}", backup_path.display());
            println!(
                "Created: {}",
                validation.backup_date.format("%Y-%m-%d %H:%M:%S UTC")
            );
            println!("Schema version: {}", validation.schema_version);
            println!("Status: {}", validation.summary());
            println!();

            if !force {
                println!("WARNING: This will overwrite ALL current data!");
                println!("To proceed, run again with --force flag:");
                println!("  envelope backup restore {} --force", backup);
                return Ok(());
            }

            // Create a backup of current data before restoring
            println!("Creating backup of current data before restore...");
            let pre_restore_backup = manager.create_backup()?;
            println!(
                "Pre-restore backup saved: {}",
                pre_restore_backup.file_name().unwrap().to_string_lossy()
            );
            println!();

            println!("Restoring from backup...");
            let result = restore_manager.restore_from_file(&backup_path)?;

            println!("Restore complete!");
            println!("{}", result.summary());

            if result.all_restored() {
                println!("\nAll data has been restored successfully.");
            } else {
                println!("\nNote: Some data may not have been present in the backup.");
            }
        }

        BackupCommands::Info { backup } => {
            let backup_path = resolve_backup_path(&manager, paths, &backup)?;

            let restore_manager = RestoreManager::new(paths.clone());
            let validation = restore_manager.validate_backup(&backup_path)?;

            let metadata = std::fs::metadata(&backup_path)?;

            println!("Backup Details");
            println!("==============");
            println!("File: {}", backup_path.display());
            println!("Size: {}", format_size(metadata.len()));
            println!(
                "Created: {}",
                validation.backup_date.format("%Y-%m-%d %H:%M:%S UTC")
            );
            println!("Schema version: {}", validation.schema_version);
            println!();
            println!("Contents:");
            println!(
                "  Accounts:     {}",
                if validation.has_accounts { "Yes" } else { "No" }
            );
            println!(
                "  Transactions: {}",
                if validation.has_transactions {
                    "Yes"
                } else {
                    "No"
                }
            );
            println!(
                "  Budget:       {}",
                if validation.has_budget { "Yes" } else { "No" }
            );
            println!(
                "  Payees:       {}",
                if validation.has_payees { "Yes" } else { "No" }
            );
            println!();
            println!(
                "Status: {}",
                if validation.is_complete() {
                    "Complete"
                } else {
                    "Partial"
                }
            );
        }

        BackupCommands::Prune { force } => {
            let backups = manager.list_backups()?;
            let retention = settings.backup_retention.clone();

            // Calculate how many would be deleted
            let (monthly, daily): (Vec<_>, Vec<_>) = backups.iter().partition(|b| b.is_monthly);

            let daily_to_delete = daily.len().saturating_sub(retention.daily_count as usize);
            let monthly_to_delete = monthly
                .len()
                .saturating_sub(retention.monthly_count as usize);
            let total_to_delete = daily_to_delete + monthly_to_delete;

            if total_to_delete == 0 {
                println!("No backups to prune.");
                println!(
                    "Current retention policy: {} daily, {} monthly",
                    retention.daily_count, retention.monthly_count
                );
                println!(
                    "You have {} daily and {} monthly backups.",
                    daily.len(),
                    monthly.len()
                );
                return Ok(());
            }

            println!("Prune Summary");
            println!("=============");
            println!(
                "Retention policy: {} daily, {} monthly",
                retention.daily_count, retention.monthly_count
            );
            println!(
                "Current backups: {} daily, {} monthly",
                daily.len(),
                monthly.len()
            );
            println!(
                "To be deleted: {} daily, {} monthly ({} total)",
                daily_to_delete, monthly_to_delete, total_to_delete
            );
            println!();

            if !force {
                println!("To delete old backups, run again with --force flag:");
                println!("  envelope backup prune --force");
                return Ok(());
            }

            let deleted = manager.enforce_retention()?;
            println!("Deleted {} backup(s).", deleted.len());
        }
    }

    Ok(())
}

/// Resolve a backup identifier to a full path
fn resolve_backup_path(
    manager: &BackupManager,
    paths: &EnvelopePaths,
    backup: &str,
) -> EnvelopeResult<PathBuf> {
    // Handle "latest" keyword
    if backup.eq_ignore_ascii_case("latest") {
        return manager.get_latest_backup()?.map(|b| b.path).ok_or_else(|| {
            crate::error::EnvelopeError::NotFound {
                entity_type: "Backup",
                identifier: "latest".to_string(),
            }
        });
    }

    // Check if it's a full path
    let path = PathBuf::from(backup);
    if path.exists() {
        return Ok(path);
    }

    // Check if it's a filename in the backup directory
    let backup_path = paths.backup_dir().join(backup);
    if backup_path.exists() {
        return Ok(backup_path);
    }

    // Try adding common backup extensions
    for ext in &["json", "yaml", "yml"] {
        let with_ext = paths.backup_dir().join(format!("{}.{}", backup, ext));
        if with_ext.exists() {
            return Ok(with_ext);
        }
    }

    Err(crate::error::EnvelopeError::NotFound {
        entity_type: "Backup",
        identifier: backup.to_string(),
    })
}

/// Format a duration in human-readable form
fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();

    if total_seconds < 60 {
        return format!("{}s", total_seconds);
    }

    let minutes = total_seconds / 60;
    if minutes < 60 {
        return format!("{}m", minutes);
    }

    let hours = minutes / 60;
    if hours < 24 {
        return format!("{}h", hours);
    }

    let days = hours / 24;
    if days < 30 {
        return format!("{}d", days);
    }

    let months = days / 30;
    format!("{}mo", months)
}

/// Format a file size in human-readable form
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
