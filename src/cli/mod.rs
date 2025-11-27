//! CLI command handlers
//!
//! This module contains the implementation of CLI commands,
//! bridging the clap argument parsing with the service layer.

pub mod account;
pub mod backup;
pub mod budget;
pub mod category;

pub use account::{handle_account_command, AccountCommands};
pub use backup::{handle_backup_command, BackupCommands};
pub use budget::{handle_budget_command, BudgetCommands};
pub use category::{handle_category_command, CategoryCommands};
