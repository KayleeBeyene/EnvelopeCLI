//! CLI command handlers
//!
//! This module contains the implementation of CLI commands,
//! bridging the clap argument parsing with the service layer.

pub mod account;
pub mod backup;
pub mod budget;
pub mod category;
pub mod encrypt;
pub mod export;
pub mod import;
pub mod payee;
pub mod reconcile;
pub mod report;
pub mod transaction;
pub mod transfer;

pub use account::{handle_account_command, AccountCommands};
pub use backup::{handle_backup_command, BackupCommands};
pub use budget::{handle_budget_command, BudgetCommands};
pub use category::{handle_category_command, CategoryCommands};
pub use encrypt::{handle_encrypt_command, EncryptCommands};
pub use export::{handle_export_command, ExportCommands};
pub use import::handle_import_command;
pub use payee::{handle_payee_command, PayeeCommands};
pub use reconcile::{handle_reconcile_command, ReconcileCommands};
pub use report::{handle_report_command, ReportCommands};
pub use transaction::{handle_transaction_command, TransactionCommands};
pub use transfer::handle_transfer_command;
