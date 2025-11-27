use anyhow::Result;
use clap::{Parser, Subcommand};

use envelope::config::{paths::EnvelopePaths, settings::Settings};

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
    Account(AccountCommands),

    /// Category management commands
    #[command(subcommand)]
    Category(CategoryCommands),

    /// Budget management commands
    #[command(subcommand)]
    Budget(BudgetCommands),

    /// Transaction management commands
    #[command(subcommand, alias = "txn")]
    Transaction(TransactionCommands),

    /// Initialize a new budget
    Init,

    /// Show current configuration and paths
    Config,
}

#[derive(Subcommand)]
enum AccountCommands {
    /// Create a new account
    Create {
        /// Account name
        name: String,
        /// Account type (checking, savings, credit, cash, investment)
        #[arg(short, long, default_value = "checking")]
        account_type: String,
        /// Starting balance in cents (e.g., 10000 for $100.00)
        #[arg(short, long, default_value = "0")]
        balance: i64,
        /// Mark as off-budget
        #[arg(long)]
        off_budget: bool,
    },
    /// List all accounts
    List {
        /// Show archived accounts
        #[arg(short, long)]
        all: bool,
    },
    /// Show account details
    Show {
        /// Account name or ID
        account: String,
    },
    /// Edit an account
    Edit {
        /// Account name or ID
        account: String,
        /// New name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Archive an account
    Archive {
        /// Account name or ID
        account: String,
    },
}

#[derive(Subcommand)]
enum CategoryCommands {
    /// Create a new category
    Create {
        /// Category name
        name: String,
        /// Category group name
        #[arg(short, long)]
        group: String,
    },
    /// List all categories
    List,
    /// Create a new category group
    #[command(name = "create-group")]
    CreateGroup {
        /// Group name
        name: String,
    },
}

#[derive(Subcommand)]
enum BudgetCommands {
    /// Assign funds to a category
    Assign {
        /// Category name
        category: String,
        /// Amount in cents (e.g., 10000 for $100.00)
        amount: i64,
        /// Budget period (e.g., 2025-01)
        #[arg(short, long)]
        period: Option<String>,
    },
    /// Move funds between categories
    Move {
        /// Source category
        from: String,
        /// Destination category
        to: String,
        /// Amount in cents
        amount: i64,
    },
    /// Show budget overview
    Overview {
        /// Budget period
        #[arg(short, long)]
        period: Option<String>,
    },
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

    match cli.command {
        Some(Commands::Tui) => {
            println!("TUI mode not yet implemented. Coming in Phase 4!");
        }
        Some(Commands::Account(cmd)) => handle_account_command(cmd),
        Some(Commands::Category(cmd)) => handle_category_command(cmd),
        Some(Commands::Budget(cmd)) => handle_budget_command(cmd),
        Some(Commands::Transaction(cmd)) => handle_transaction_command(cmd),
        Some(Commands::Init) => {
            println!("Initializing EnvelopeCLI at: {}", paths.data_dir().display());
            paths.ensure_directories()?;
            settings.save(&paths)?;
            println!("Initialization complete!");
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

fn handle_account_command(cmd: AccountCommands) {
    match cmd {
        AccountCommands::Create { name, account_type, balance, off_budget } => {
            println!("Creating account '{}' (type: {}, balance: {}, off-budget: {})",
                     name, account_type, balance, off_budget);
            println!("Account management will be implemented in Phase 2, Step 6.");
        }
        AccountCommands::List { all } => {
            println!("Listing accounts (show archived: {})", all);
            println!("Account management will be implemented in Phase 2, Step 6.");
        }
        AccountCommands::Show { account } => {
            println!("Showing account: {}", account);
            println!("Account management will be implemented in Phase 2, Step 6.");
        }
        AccountCommands::Edit { account, name } => {
            println!("Editing account: {} (new name: {:?})", account, name);
            println!("Account management will be implemented in Phase 2, Step 6.");
        }
        AccountCommands::Archive { account } => {
            println!("Archiving account: {}", account);
            println!("Account management will be implemented in Phase 2, Step 6.");
        }
    }
}

fn handle_category_command(cmd: CategoryCommands) {
    match cmd {
        CategoryCommands::Create { name, group } => {
            println!("Creating category '{}' in group '{}'", name, group);
            println!("Category management will be implemented in Phase 2, Step 7.");
        }
        CategoryCommands::List => {
            println!("Listing categories");
            println!("Category management will be implemented in Phase 2, Step 7.");
        }
        CategoryCommands::CreateGroup { name } => {
            println!("Creating category group '{}'", name);
            println!("Category management will be implemented in Phase 2, Step 7.");
        }
    }
}

fn handle_budget_command(cmd: BudgetCommands) {
    match cmd {
        BudgetCommands::Assign { category, amount, period } => {
            println!("Assigning {} to '{}' (period: {:?})", amount, category, period);
            println!("Budget management will be implemented in Phase 2, Step 9.");
        }
        BudgetCommands::Move { from, to, amount } => {
            println!("Moving {} from '{}' to '{}'", amount, from, to);
            println!("Budget management will be implemented in Phase 2, Step 9.");
        }
        BudgetCommands::Overview { period } => {
            println!("Budget overview (period: {:?})", period);
            println!("Budget management will be implemented in Phase 2, Step 9.");
        }
    }
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
