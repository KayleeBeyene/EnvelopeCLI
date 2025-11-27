//! Reconciliation CLI commands
//!
//! Implements CLI commands for account reconciliation workflow.

use chrono::NaiveDate;
use clap::Subcommand;

use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::Money;
use crate::services::{AccountService, CategoryService, ReconciliationService};
use crate::storage::Storage;

/// Reconciliation subcommands
#[derive(Subcommand)]
pub enum ReconcileCommands {
    /// Start reconciliation for an account
    Start {
        /// Account name or ID
        account: String,
        /// Statement ending balance (e.g., "1234.56")
        balance: String,
        /// Statement date (YYYY-MM-DD), defaults to today
        #[arg(short, long)]
        date: Option<String>,
    },
    /// Show reconciliation status for an account
    Status {
        /// Account name or ID
        account: String,
        /// Statement ending balance (e.g., "1234.56")
        balance: String,
        /// Statement date (YYYY-MM-DD), defaults to today
        #[arg(short, long)]
        date: Option<String>,
    },
    /// Mark a transaction as cleared during reconciliation
    Clear {
        /// Transaction ID
        id: String,
    },
    /// Mark a transaction as pending during reconciliation
    Unclear {
        /// Transaction ID
        id: String,
    },
    /// Complete reconciliation (requires difference to be zero)
    Complete {
        /// Account name or ID
        account: String,
        /// Statement ending balance (e.g., "1234.56")
        balance: String,
        /// Statement date (YYYY-MM-DD), defaults to today
        #[arg(short, long)]
        date: Option<String>,
    },
    /// Complete reconciliation with adjustment for discrepancies
    Adjust {
        /// Account name or ID
        account: String,
        /// Statement ending balance (e.g., "1234.56")
        balance: String,
        /// Statement date (YYYY-MM-DD), defaults to today
        #[arg(short, long)]
        date: Option<String>,
        /// Category for the adjustment transaction
        #[arg(short, long)]
        category: Option<String>,
    },
}

/// Handle a reconcile command
pub fn handle_reconcile_command(storage: &Storage, cmd: ReconcileCommands) -> EnvelopeResult<()> {
    let service = ReconciliationService::new(storage);
    let account_service = AccountService::new(storage);
    let category_service = CategoryService::new(storage);

    match cmd {
        ReconcileCommands::Start { account, balance, date } => {
            let account = account_service
                .find(&account)?
                .ok_or_else(|| EnvelopeError::account_not_found(&account))?;

            let statement_balance = Money::parse(&balance).map_err(|e| {
                EnvelopeError::Validation(format!(
                    "Invalid balance format: '{}'. Use format like '1234.56'. Error: {}",
                    balance, e
                ))
            })?;

            let statement_date = parse_date_or_today(date.as_deref())?;

            let session = service.start(account.id, statement_date, statement_balance)?;
            let summary = service.get_summary(&session)?;

            println!("Reconciliation started for: {}", account.name);
            println!("Statement Date: {}", statement_date);
            println!("Statement Balance: {}", statement_balance);
            println!();
            println!("Current Status:");
            println!("  Starting reconciled balance: {}", session.starting_cleared_balance);
            println!("  Current cleared balance:     {}", summary.current_cleared_balance);
            println!("  Difference:                  {}", summary.difference);
            println!();
            println!("Transactions:");
            println!("  Cleared (ready to reconcile): {}", summary.cleared_transactions.len());
            println!("  Pending (uncleared):          {}", summary.uncleared_transactions.len());

            if !summary.cleared_transactions.is_empty() {
                println!();
                println!("Cleared transactions:");
                for txn in &summary.cleared_transactions {
                    println!(
                        "  {} {} {:>12}  {}",
                        txn.id.to_string().chars().take(8).collect::<String>(),
                        txn.date,
                        txn.amount,
                        txn.payee_name
                    );
                }
            }

            if !summary.uncleared_transactions.is_empty() {
                println!();
                println!("Pending transactions:");
                for txn in &summary.uncleared_transactions {
                    println!(
                        "  {} {} {:>12}  {}",
                        txn.id.to_string().chars().take(8).collect::<String>(),
                        txn.date,
                        txn.amount,
                        txn.payee_name
                    );
                }
            }

            println!();
            if summary.can_complete {
                println!("Ready to complete! Run 'envelope reconcile complete' to finish.");
            } else {
                println!("Difference is {}. Clear/unclear transactions until difference is $0.00", summary.difference);
                println!("Or use 'envelope reconcile adjust' to create an adjustment transaction.");
            }
        }

        ReconcileCommands::Status { account, balance, date } => {
            let account = account_service
                .find(&account)?
                .ok_or_else(|| EnvelopeError::account_not_found(&account))?;

            let statement_balance = Money::parse(&balance).map_err(|e| {
                EnvelopeError::Validation(format!("Invalid balance: {}", e))
            })?;

            let statement_date = parse_date_or_today(date.as_deref())?;

            let session = service.start(account.id, statement_date, statement_balance)?;
            let summary = service.get_summary(&session)?;

            println!("Reconciliation Status: {}", account.name);
            println!("{}", "=".repeat(40));
            println!();
            println!("Statement balance:     {}", statement_balance);
            println!("Current cleared:       {}", summary.current_cleared_balance);
            println!("Difference:            {}", summary.difference);
            println!();

            if let Some(last_date) = account.last_reconciled_date {
                println!("Last reconciliation:   {}", last_date);
                if let Some(last_balance) = account.last_reconciled_balance {
                    println!("Last reconciled balance: {}", last_balance);
                }
            } else {
                println!("Last reconciliation:   Never");
            }
        }

        ReconcileCommands::Clear { id } => {
            let txn = service.clear_transaction(id.parse().map_err(|_| {
                EnvelopeError::Validation(format!("Invalid transaction ID: {}", id))
            })?)?;

            println!(
                "Cleared: {} {} {}",
                txn.date, txn.payee_name, txn.amount
            );
        }

        ReconcileCommands::Unclear { id } => {
            let txn = service.unclear_transaction(id.parse().map_err(|_| {
                EnvelopeError::Validation(format!("Invalid transaction ID: {}", id))
            })?)?;

            println!(
                "Uncleared: {} {} {}",
                txn.date, txn.payee_name, txn.amount
            );
        }

        ReconcileCommands::Complete { account, balance, date } => {
            let account = account_service
                .find(&account)?
                .ok_or_else(|| EnvelopeError::account_not_found(&account))?;

            let statement_balance = Money::parse(&balance).map_err(|e| {
                EnvelopeError::Validation(format!("Invalid balance: {}", e))
            })?;

            let statement_date = parse_date_or_today(date.as_deref())?;

            let session = service.start(account.id, statement_date, statement_balance)?;
            let result = service.complete(&session)?;

            println!("Reconciliation complete!");
            println!("  Account: {}", account.name);
            println!("  Statement date: {}", statement_date);
            println!("  Statement balance: {}", statement_balance);
            println!("  Transactions reconciled: {}", result.transactions_reconciled);
        }

        ReconcileCommands::Adjust { account, balance, date, category } => {
            let account = account_service
                .find(&account)?
                .ok_or_else(|| EnvelopeError::account_not_found(&account))?;

            let statement_balance = Money::parse(&balance).map_err(|e| {
                EnvelopeError::Validation(format!("Invalid balance: {}", e))
            })?;

            let statement_date = parse_date_or_today(date.as_deref())?;

            let category_id = if let Some(cat_name) = category {
                let cat = category_service
                    .find_category(&cat_name)?
                    .ok_or_else(|| EnvelopeError::category_not_found(&cat_name))?;
                Some(cat.id)
            } else {
                None
            };

            let session = service.start(account.id, statement_date, statement_balance)?;
            let summary = service.get_summary(&session)?;

            println!("Creating adjustment transaction for: {}", summary.difference);

            let result = service.complete_with_adjustment(&session, category_id)?;

            println!();
            println!("Reconciliation complete with adjustment!");
            println!("  Account: {}", account.name);
            println!("  Statement date: {}", statement_date);
            println!("  Statement balance: {}", statement_balance);
            println!("  Transactions reconciled: {}", result.transactions_reconciled);
            if result.adjustment_created {
                println!("  Adjustment created: {}", result.adjustment_amount.unwrap());
            }
        }
    }

    Ok(())
}

/// Parse a date string or return today's date
fn parse_date_or_today(date_str: Option<&str>) -> EnvelopeResult<NaiveDate> {
    if let Some(date_str) = date_str {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            EnvelopeError::Validation(format!(
                "Invalid date format: '{}'. Use YYYY-MM-DD",
                date_str
            ))
        })
    } else {
        Ok(chrono::Local::now().date_naive())
    }
}
