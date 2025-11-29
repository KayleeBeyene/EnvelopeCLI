use anyhow::Result;
use clap::{Parser, Subcommand};

use envelope_cli::cli::{
    handle_account_command, handle_backup_command, handle_budget_command, handle_category_command,
    handle_encrypt_command, handle_export_command, handle_import_command, handle_income_command,
    handle_payee_command, handle_reconcile_command, handle_report_command, handle_target_command,
    handle_transaction_command, handle_transfer_command,
};
use envelope_cli::config::{paths::EnvelopePaths, settings::Settings};
use envelope_cli::storage::Storage;

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
    Account(envelope_cli::cli::AccountCommands),

    /// Category management commands
    #[command(subcommand)]
    Category(envelope_cli::cli::CategoryCommands),

    /// Budget management commands
    #[command(subcommand)]
    Budget(envelope_cli::cli::BudgetCommands),

    /// Budget target management commands
    #[command(subcommand)]
    Target(envelope_cli::cli::TargetCommands),

    /// Expected income management commands
    #[command(subcommand)]
    Income(envelope_cli::cli::IncomeCommands),

    /// Backup management commands
    #[command(subcommand)]
    Backup(envelope_cli::cli::BackupCommands),

    /// Transaction management commands
    #[command(subcommand, alias = "txn")]
    Transaction(envelope_cli::cli::TransactionCommands),

    /// Payee management commands
    #[command(subcommand)]
    Payee(envelope_cli::cli::PayeeCommands),

    /// Reconciliation commands
    #[command(subcommand)]
    Reconcile(envelope_cli::cli::ReconcileCommands),

    /// Generate reports
    #[command(subcommand)]
    Report(envelope_cli::cli::ReportCommands),

    /// Export data
    #[command(subcommand)]
    Export(envelope_cli::cli::ExportCommands),

    /// Encryption management commands
    #[command(subcommand)]
    Encrypt(envelope_cli::cli::EncryptCommands),

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
    let mut settings = Settings::load_or_create(&paths)?;

    // Initialize storage
    let mut storage = Storage::new(paths.clone())?;
    storage.load_all()?;

    match cli.command {
        Some(Commands::Tui) => {
            // Launch the TUI
            envelope_cli::tui::run_tui(&storage, &settings, &paths)?;
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
        Some(Commands::Target(cmd)) => {
            handle_target_command(&storage, &settings, cmd)?;
        }
        Some(Commands::Income(cmd)) => {
            handle_income_command(&storage, &settings, cmd)?;
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
        Some(Commands::Report(cmd)) => {
            handle_report_command(&storage, cmd)?;
        }
        Some(Commands::Export(cmd)) => {
            handle_export_command(&storage, cmd)?;
        }
        Some(Commands::Encrypt(cmd)) => {
            handle_encrypt_command(&paths, &mut settings, &storage, cmd)?;
        }
        Some(Commands::Transfer {
            from,
            to,
            amount,
            date,
            memo,
        }) => {
            handle_transfer_command(&storage, &from, &to, &amount, date.as_deref(), memo)?;
        }
        Some(Commands::Import { file, account }) => {
            handle_import_command(&storage, &file, &account)?;
        }
        Some(Commands::Init) => {
            println!(
                "Initializing EnvelopeCLI at: {}",
                paths.data_dir().display()
            );
            envelope_cli::storage::init::initialize_storage(&paths)?;
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
            println!("  Encryption enabled: {}", settings.is_encryption_enabled());
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
