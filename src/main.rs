use anyhow::Result;
use clap::{Parser, Subcommand};

use envelope::cli::{
    handle_account_command, handle_backup_command, handle_budget_command, handle_category_command,
    handle_payee_command, handle_reconcile_command, handle_transaction_command,
};
use envelope::config::{paths::EnvelopePaths, settings::Settings};
use envelope::storage::Storage;

#[derive(Parser)]
#[command(
    name = "envelope",
    author = "Kaylee Beyene",
    version,
    about = "Terminal-based zero-based budgeting application",
    long_about = "EnvelopeCLI is a terminal-based zero-based budgeting application \
                  inspired by YNAB. It helps you give every dollar a job and take \
                  control of your finances from the command line."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the interactive TUI
    #[command(alias = "ui")]
    Tui,

    /// Account management commands
    #[command(subcommand)]
    Account(envelope::cli::AccountCommands),

    /// Category management commands
    #[command(subcommand)]
    Category(envelope::cli::CategoryCommands),

    /// Budget management commands
    #[command(subcommand)]
    Budget(envelope::cli::BudgetCommands),

    /// Backup management commands
    #[command(subcommand)]
    Backup(envelope::cli::BackupCommands),

    /// Transaction management commands
    #[command(subcommand, alias = "txn")]
    Transaction(envelope::cli::TransactionCommands),

    /// Payee management commands
    #[command(subcommand)]
    Payee(envelope::cli::PayeeCommands),

    /// Reconciliation commands
    #[command(subcommand)]
    Reconcile(envelope::cli::ReconcileCommands),

    /// Transfer between accounts
    Transfer {
        /// Source account name
        from: String,
        /// Destination account name
        to: String,
        /// Amount to transfer (e.g., "100.00")
        amount: String,
        /// Transaction date (YYYY-MM-DD), defaults to today
        #[arg(short, long)]
        date: Option<String>,
        /// Memo
        #[arg(short, long)]
        memo: Option<String>,
    },

    /// Import transactions from CSV
    Import {
        /// Path to CSV file
        file: String,
        /// Target account name or ID
        #[arg(short, long)]
        account: String,
    },

    /// Initialize a new budget
    Init,

    /// Show current configuration and paths
    Config,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize paths and settings
    let paths = EnvelopePaths::new()?;
    let settings = Settings::load_or_create(&paths)?;

    // Initialize storage
    let mut storage = Storage::new(paths.clone())?;
    storage.load_all()?;

    match cli.command {
        Some(Commands::Tui) => {
            // Launch the TUI
            envelope::tui::run_tui(&storage, &settings, &paths)?;
        }
        Some(Commands::Account(cmd)) => {
            handle_account_command(&storage, cmd)?;
        }
        Some(Commands::Category(cmd)) => {
            handle_category_command(&storage, cmd)?;
        }
        Some(Commands::Budget(cmd)) => {
            handle_budget_command(&storage, &settings, cmd)?;
        }
        Some(Commands::Backup(cmd)) => {
            handle_backup_command(&paths, &settings, cmd)?;
        }
        Some(Commands::Transaction(cmd)) => {
            handle_transaction_command(&storage, cmd)?;
        }
        Some(Commands::Payee(cmd)) => {
            handle_payee_command(&storage, cmd)?;
        }
        Some(Commands::Reconcile(cmd)) => {
            handle_reconcile_command(&storage, cmd)?;
        }
        Some(Commands::Transfer {
            from,
            to,
            amount,
            date,
            memo,
        }) => {
            use envelope::services::{AccountService, TransferService};
            use envelope::models::Money;
            use chrono::NaiveDate;

            let account_service = AccountService::new(&storage);
            let transfer_service = TransferService::new(&storage);

            // Find source account
            let from_account = account_service.find(&from)?.ok_or_else(|| {
                envelope::error::EnvelopeError::account_not_found(&from)
            })?;

            // Find destination account
            let to_account = account_service.find(&to)?.ok_or_else(|| {
                envelope::error::EnvelopeError::account_not_found(&to)
            })?;

            // Parse amount
            let amount = Money::parse(&amount).map_err(|e| {
                envelope::error::EnvelopeError::Validation(format!(
                    "Invalid amount format: '{}'. Use format like '100.00' or '100'. Error: {}",
                    amount, e
                ))
            })?;

            // Parse date (default to today)
            let date = if let Some(date_str) = date {
                NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|_| {
                    envelope::error::EnvelopeError::Validation(format!(
                        "Invalid date format: '{}'. Use YYYY-MM-DD",
                        date_str
                    ))
                })?
            } else {
                chrono::Local::now().date_naive()
            };

            let result = transfer_service.create_transfer(
                from_account.id,
                to_account.id,
                amount,
                date,
                memo,
            )?;

            println!("Transfer created:");
            println!("  From: {} ({})", from_account.name, result.from_transaction.amount);
            println!("  To:   {} ({})", to_account.name, result.to_transaction.amount);
            println!("  Date: {}", date);
        }
        Some(Commands::Import { file, account }) => {
            use envelope::services::{AccountService, ImportService, ImportStatus};
            use std::path::Path;

            let account_service = AccountService::new(&storage);
            let import_service = ImportService::new(&storage);

            // Find account
            let target_account = account_service.find(&account)?.ok_or_else(|| {
                envelope::error::EnvelopeError::account_not_found(&account)
            })?;

            let path = Path::new(&file);
            if !path.exists() {
                return Err(envelope::error::EnvelopeError::Import(
                    format!("File not found: {}", file)
                ).into());
            }

            // Try to detect mapping from CSV header
            let content = std::fs::read_to_string(path)?;
            let first_line = content.lines().next().unwrap_or("");
            let mapping = import_service.detect_mapping(first_line);

            // Parse the CSV
            let parsed = import_service.parse_csv(&content, &mapping)?;

            if parsed.is_empty() {
                println!("No transactions found in CSV file.");
                return Ok(());
            }

            // Generate preview
            let preview = import_service.generate_preview(&parsed, target_account.id)?;

            // Show preview summary
            let new_count = preview.iter().filter(|e| e.status == ImportStatus::New).count();
            let dup_count = preview.iter().filter(|e| e.status == ImportStatus::Duplicate).count();
            let err_count = preview.iter().filter(|e| matches!(e.status, ImportStatus::Error(_))).count();

            println!("Import Preview for '{}'", target_account.name);
            println!("{}", "=".repeat(40));
            println!("  New transactions:   {}", new_count);
            println!("  Duplicates (skip):  {}", dup_count);
            println!("  Errors:             {}", err_count);
            println!();

            if new_count == 0 {
                println!("No new transactions to import.");
                return Ok(());
            }

            // Show first few new transactions
            println!("First transactions to import:");
            for entry in preview.iter().filter(|e| e.status == ImportStatus::New).take(5) {
                println!(
                    "  {} {} {}",
                    entry.transaction.date,
                    entry.transaction.payee,
                    entry.transaction.amount
                );
            }
            if new_count > 5 {
                println!("  ... and {} more", new_count - 5);
            }
            println!();

            // Perform import
            let result = import_service.import_from_preview(
                &preview,
                target_account.id,
                None, // No default category
                false, // Don't mark as cleared
            )?;

            println!("Import Complete!");
            println!("  Imported:    {}", result.imported);
            println!("  Skipped:     {}", result.duplicates_skipped);
            if !result.error_messages.is_empty() {
                println!("  Errors:      {}", result.errors);
                for (row, msg) in &result.error_messages {
                    println!("    Row {}: {}", row + 1, msg);
                }
            }
        }
        Some(Commands::Init) => {
            println!("Initializing EnvelopeCLI at: {}", paths.data_dir().display());
            envelope::storage::init::initialize_storage(&paths)?;
            settings.save(&paths)?;
            println!("Initialization complete!");
            println!();
            println!("Default category groups and categories have been created:");
            println!("  - Bills (Rent/Mortgage, Electric, Water, Internet, Phone, Insurance)");
            println!("  - Needs (Groceries, Transportation, Medical, Household)");
            println!("  - Wants (Dining Out, Entertainment, Shopping, Subscriptions)");
            println!("  - Savings (Emergency Fund, Vacation, Large Purchases)");
            println!();
            println!("Run 'envelope category list' to see all categories.");
        }
        Some(Commands::Config) => {
            println!("EnvelopeCLI Configuration");
            println!("========================");
            println!("Config directory: {}", paths.config_dir().display());
            println!("Data directory:   {}", paths.data_dir().display());
            println!("Backup directory: {}", paths.backup_dir().display());
            println!();
            println!("Settings:");
            println!("  Budget period type: {:?}", settings.budget_period_type);
            println!("  Encryption enabled: {}", settings.encryption_enabled);
        }
        None => {
            println!("EnvelopeCLI - Terminal-based zero-based budgeting");
            println!();
            println!("Run 'envelope --help' for usage information.");
            println!("Run 'envelope tui' to launch the interactive interface.");
        }
    }

    Ok(())
}
