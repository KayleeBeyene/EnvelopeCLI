use anyhow::Result;
use clap::{Parser, Subcommand};

use envelope::cli::{handle_account_command, handle_budget_command, handle_category_command};
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

    /// Transaction management commands
    #[command(subcommand, alias = "txn")]
    Transaction(TransactionCommands),

    /// Initialize a new budget
    Init,

    /// Show current configuration and paths
    Config,
}




#[derive(Subcommand)]
enum TransactionCommands {
    /// Add a new transaction
    Add {
        /// Account name or ID
        account: String,
        /// Amount in cents (negative for outflow)
        amount: i64,
        /// Payee name
        #[arg(short, long)]
        payee: Option<String>,
        /// Category name
        #[arg(short, long)]
        category: Option<String>,
        /// Transaction date (YYYY-MM-DD)
        #[arg(short, long)]
        date: Option<String>,
        /// Memo
        #[arg(short, long)]
        memo: Option<String>,
    },
    /// List transactions
    List {
        /// Filter by account
        #[arg(short, long)]
        account: Option<String>,
        /// Number of transactions to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Import transactions from CSV
    Import {
        /// Path to CSV file
        file: String,
        /// Target account
        #[arg(short, long)]
        account: String,
    },
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
            println!("TUI mode not yet implemented. Coming in Phase 4!");
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
        Some(Commands::Transaction(cmd)) => handle_transaction_command(cmd),
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




fn handle_transaction_command(cmd: TransactionCommands) {
    match cmd {
        TransactionCommands::Add { account, amount, payee, category, date, memo } => {
            println!("Adding transaction to '{}': {} (payee: {:?}, category: {:?}, date: {:?}, memo: {:?})",
                     account, amount, payee, category, date, memo);
            println!("Transaction management will be implemented in Phase 3, Step 11.");
        }
        TransactionCommands::List { account, limit } => {
            println!("Listing transactions (account: {:?}, limit: {})", account, limit);
            println!("Transaction management will be implemented in Phase 3, Step 11.");
        }
        TransactionCommands::Import { file, account } => {
            println!("Importing from '{}' to account '{}'", file, account);
            println!("CSV import will be implemented in Phase 3, Steps 15-16.");
        }
    }
}
