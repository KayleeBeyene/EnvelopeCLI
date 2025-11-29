//! Account CLI commands
//!
//! Implements CLI commands for account management.

use clap::Subcommand;

use crate::display::account::{format_account_details, format_account_list};
use crate::error::EnvelopeResult;
use crate::models::{AccountType, Money};
use crate::services::AccountService;
use crate::storage::Storage;

/// Account subcommands
#[derive(Subcommand)]
pub enum AccountCommands {
    /// Create a new account
    Create {
        /// Account name
        name: String,
        /// Account type (checking, savings, credit, cash, investment)
        #[arg(short = 't', long, default_value = "checking")]
        account_type: String,
        /// Starting balance (e.g., "1000.00" or "1000")
        #[arg(short, long, default_value = "0")]
        balance: String,
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
    /// Unarchive an account
    Unarchive {
        /// Account name or ID
        account: String,
    },
}

/// Handle an account command
pub fn handle_account_command(storage: &Storage, cmd: AccountCommands) -> EnvelopeResult<()> {
    let service = AccountService::new(storage);

    match cmd {
        AccountCommands::Create {
            name,
            account_type,
            balance,
            off_budget,
        } => {
            let account_type = AccountType::parse(&account_type).ok_or_else(|| {
                crate::error::EnvelopeError::Validation(format!(
                    "Invalid account type: '{}'. Valid types: checking, savings, credit, cash, investment, line_of_credit, other",
                    account_type
                ))
            })?;

            let mut starting_balance = Money::parse(&balance).map_err(|e| {
                crate::error::EnvelopeError::Validation(format!(
                    "Invalid balance format: '{}'. Use format like '1000.00' or '1000'. Error: {}",
                    balance, e
                ))
            })?;

            // For liability accounts (credit cards, lines of credit), balances represent
            // debt owed and should be stored as negative values. Users naturally enter
            // positive numbers when specifying debt, so we negate them.
            if account_type.is_liability() && starting_balance.cents() > 0 {
                starting_balance = Money::from_cents(-starting_balance.cents());
            }

            let account = service.create(&name, account_type, starting_balance, !off_budget)?;

            println!("Created account: {}", account.name);
            println!("  Type: {}", account.account_type);
            println!("  Starting Balance: {}", account.starting_balance);
            println!(
                "  On Budget: {}",
                if account.on_budget { "Yes" } else { "No" }
            );
            println!("  ID: {}", account.id);
        }

        AccountCommands::List { all } => {
            let summaries = service.list_with_balances(all)?;
            print!("{}", format_account_list(&summaries));
        }

        AccountCommands::Show { account } => {
            let found = service
                .find(&account)?
                .ok_or_else(|| crate::error::EnvelopeError::account_not_found(&account))?;

            let summary = service.get_summary(&found)?;
            print!("{}", format_account_details(&summary));
        }

        AccountCommands::Edit { account, name } => {
            let found = service
                .find(&account)?
                .ok_or_else(|| crate::error::EnvelopeError::account_not_found(&account))?;

            if name.is_none() {
                println!("No changes specified. Use --name to change the account name.");
                return Ok(());
            }

            let updated = service.update(found.id, name.as_deref())?;
            println!("Updated account: {}", updated.name);
        }

        AccountCommands::Archive { account } => {
            let found = service
                .find(&account)?
                .ok_or_else(|| crate::error::EnvelopeError::account_not_found(&account))?;

            let archived = service.archive(found.id)?;
            println!("Archived account: {}", archived.name);
        }

        AccountCommands::Unarchive { account } => {
            let found = service
                .find(&account)?
                .ok_or_else(|| crate::error::EnvelopeError::account_not_found(&account))?;

            let unarchived = service.unarchive(found.id)?;
            println!("Unarchived account: {}", unarchived.name);
        }
    }

    Ok(())
}
