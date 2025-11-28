//! Transaction CLI commands
//!
//! Implements CLI commands for transaction management.

use chrono::NaiveDate;
use clap::Subcommand;

use crate::display::transaction::{
    format_transaction_details, format_transaction_list_by_account, format_transaction_register,
};
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{Money, TransactionStatus};
use crate::services::{
    AccountService, CategoryService, CreateTransactionInput, PayeeService, TransactionFilter,
    TransactionService,
};
use crate::storage::Storage;

/// Transaction subcommands
#[derive(Subcommand)]
pub enum TransactionCommands {
    /// Add a new transaction
    Add {
        /// Account name or ID
        account: String,
        /// Amount (e.g., "-50.00" for outflow, "100.00" for inflow)
        amount: String,
        /// Payee name
        #[arg(short, long)]
        payee: Option<String>,
        /// Category name
        #[arg(short, long)]
        category: Option<String>,
        /// Transaction date (YYYY-MM-DD), defaults to today
        #[arg(short, long)]
        date: Option<String>,
        /// Memo
        #[arg(short, long)]
        memo: Option<String>,
        /// Mark as cleared
        #[arg(long)]
        cleared: bool,
        /// Auto-categorize based on payee history
        #[arg(long)]
        auto_categorize: bool,
    },
    /// List transactions
    List {
        /// Filter by account name or ID
        #[arg(short, long)]
        account: Option<String>,
        /// Filter by category name
        #[arg(short = 'C', long)]
        category: Option<String>,
        /// Number of transactions to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,
        /// Filter by status (pending, cleared, reconciled)
        #[arg(long)]
        status: Option<String>,
    },
    /// Show transaction details
    Show {
        /// Transaction ID
        id: String,
    },
    /// Edit a transaction
    Edit {
        /// Transaction ID
        id: String,
        /// New amount
        #[arg(short, long)]
        amount: Option<String>,
        /// New payee
        #[arg(short, long)]
        payee: Option<String>,
        /// New category
        #[arg(short, long)]
        category: Option<String>,
        /// New date
        #[arg(short, long)]
        date: Option<String>,
        /// New memo
        #[arg(short, long)]
        memo: Option<String>,
    },
    /// Delete a transaction
    Delete {
        /// Transaction ID
        id: String,
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Clear a transaction (mark as cleared)
    Clear {
        /// Transaction ID
        id: String,
    },
    /// Unclear a transaction (mark as pending)
    Unclear {
        /// Transaction ID
        id: String,
    },
    /// Unlock a reconciled transaction for editing
    Unlock {
        /// Transaction ID
        id: String,
    },
}

/// Handle a transaction command
pub fn handle_transaction_command(
    storage: &Storage,
    cmd: TransactionCommands,
) -> EnvelopeResult<()> {
    let service = TransactionService::new(storage);
    let account_service = AccountService::new(storage);
    let category_service = CategoryService::new(storage);
    let payee_service = PayeeService::new(storage);

    match cmd {
        TransactionCommands::Add {
            account,
            amount,
            payee,
            category,
            date,
            memo,
            cleared,
            auto_categorize,
        } => {
            // Find account
            let account = account_service
                .find(&account)?
                .ok_or_else(|| EnvelopeError::account_not_found(&account))?;

            // Parse amount
            let amount = Money::parse(&amount).map_err(|e| {
                EnvelopeError::Validation(format!(
                    "Invalid amount format: '{}'. Use format like '-50.00' or '100'. Error: {}",
                    amount, e
                ))
            })?;

            // Parse date (default to today)
            let date = if let Some(date_str) = date {
                NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|_| {
                    EnvelopeError::Validation(format!(
                        "Invalid date format: '{}'. Use YYYY-MM-DD",
                        date_str
                    ))
                })?
            } else {
                chrono::Local::now().date_naive()
            };

            // Find category
            let mut category_id = if let Some(cat_name) = &category {
                let cat = category_service
                    .find_category(cat_name)?
                    .ok_or_else(|| EnvelopeError::category_not_found(cat_name))?;
                Some(cat.id)
            } else {
                None
            };

            // Auto-categorize from payee if requested
            if auto_categorize && category_id.is_none() {
                if let Some(payee_name) = &payee {
                    category_id = payee_service.get_suggested_category(payee_name)?;
                    if category_id.is_some() {
                        println!("Auto-categorized based on payee history");
                    }
                }
            }

            let status = if cleared {
                Some(TransactionStatus::Cleared)
            } else {
                None
            };

            let input = CreateTransactionInput {
                account_id: account.id,
                date,
                amount,
                payee_name: payee,
                category_id,
                memo,
                status,
            };

            let txn = service.create(input)?;

            // Learn from transaction (update payee category frequency)
            service.learn_from_transaction(&txn)?;

            println!("Created transaction:");
            println!("  ID:       {}", txn.id);
            println!("  Date:     {}", txn.date);
            println!("  Amount:   {}", txn.amount);
            if !txn.payee_name.is_empty() {
                println!("  Payee:    {}", txn.payee_name);
            }
            if let Some(cat_id) = txn.category_id {
                if let Some(cat) = category_service.get_category(cat_id)? {
                    println!("  Category: {}", cat.name);
                }
            }
            println!("  Status:   {}", txn.status);
        }

        TransactionCommands::List {
            account,
            category,
            limit,
            from,
            to,
            status,
        } => {
            let mut filter = TransactionFilter::new().limit(limit);

            // Apply account filter
            if let Some(acc_name) = &account {
                let acc = account_service
                    .find(acc_name)?
                    .ok_or_else(|| EnvelopeError::account_not_found(acc_name))?;
                filter = filter.account(acc.id);
            }

            // Apply category filter
            if let Some(cat_name) = &category {
                let cat = category_service
                    .find_category(cat_name)?
                    .ok_or_else(|| EnvelopeError::category_not_found(cat_name))?;
                filter = filter.category(cat.id);
            }

            // Apply date range filter
            if let Some(from_str) = from {
                let from_date = NaiveDate::parse_from_str(&from_str, "%Y-%m-%d").map_err(|_| {
                    EnvelopeError::Validation(format!(
                        "Invalid date format: '{}'. Use YYYY-MM-DD",
                        from_str
                    ))
                })?;
                filter.start_date = Some(from_date);
            }

            if let Some(to_str) = to {
                let to_date = NaiveDate::parse_from_str(&to_str, "%Y-%m-%d").map_err(|_| {
                    EnvelopeError::Validation(format!(
                        "Invalid date format: '{}'. Use YYYY-MM-DD",
                        to_str
                    ))
                })?;
                filter.end_date = Some(to_date);
            }

            // Apply status filter
            if let Some(status_str) = status {
                let status = match status_str.to_lowercase().as_str() {
                    "pending" => TransactionStatus::Pending,
                    "cleared" => TransactionStatus::Cleared,
                    "reconciled" => TransactionStatus::Reconciled,
                    _ => {
                        return Err(EnvelopeError::Validation(format!(
                            "Invalid status: '{}'. Use pending, cleared, or reconciled",
                            status_str
                        )))
                    }
                };
                filter = filter.status(status);
            }

            let transactions = service.list(filter)?;

            if let Some(acc_name) = &account {
                if let Some(acc) = account_service.find(acc_name)? {
                    print!(
                        "{}",
                        format_transaction_list_by_account(&transactions, &acc.name)
                    );
                } else {
                    print!("{}", format_transaction_register(&transactions));
                }
            } else {
                print!("{}", format_transaction_register(&transactions));
            }

            println!("\nShowing {} transactions", transactions.len());
        }

        TransactionCommands::Show { id } => {
            let txn = service
                .find(&id)?
                .ok_or_else(|| EnvelopeError::transaction_not_found(&id))?;

            let category_name = if let Some(cat_id) = txn.category_id {
                category_service.get_category(cat_id)?.map(|c| c.name)
            } else {
                None
            };

            print!(
                "{}",
                format_transaction_details(&txn, category_name.as_deref())
            );

            // Show account name
            if let Some(account) = account_service.get(txn.account_id)? {
                println!("Account:     {}", account.name);
            }
        }

        TransactionCommands::Edit {
            id,
            amount,
            payee,
            category,
            date,
            memo,
        } => {
            let txn = service
                .find(&id)?
                .ok_or_else(|| EnvelopeError::transaction_not_found(&id))?;

            // Parse new values if provided
            let new_amount = if let Some(amt_str) = amount {
                Some(
                    Money::parse(&amt_str)
                        .map_err(|e| EnvelopeError::Validation(format!("Invalid amount: {}", e)))?,
                )
            } else {
                None
            };

            let new_date = if let Some(date_str) = date {
                Some(
                    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|_| {
                        EnvelopeError::Validation(format!(
                            "Invalid date format: '{}'. Use YYYY-MM-DD",
                            date_str
                        ))
                    })?,
                )
            } else {
                None
            };

            let new_category_id = if let Some(cat_name) = category {
                if cat_name.is_empty() || cat_name.to_lowercase() == "none" {
                    // Clear category
                    Some(None)
                } else {
                    let cat = category_service
                        .find_category(&cat_name)?
                        .ok_or_else(|| EnvelopeError::category_not_found(&cat_name))?;
                    Some(Some(cat.id))
                }
            } else {
                None
            };

            let updated =
                service.update(txn.id, new_date, new_amount, payee, new_category_id, memo)?;

            println!("Updated transaction: {}", updated.id);
            println!("  Date:   {}", updated.date);
            println!("  Amount: {}", updated.amount);
            if !updated.payee_name.is_empty() {
                println!("  Payee:  {}", updated.payee_name);
            }
        }

        TransactionCommands::Delete { id, force } => {
            let txn = service
                .find(&id)?
                .ok_or_else(|| EnvelopeError::transaction_not_found(&id))?;

            if !force {
                println!("About to delete transaction:");
                println!("  Date:   {}", txn.date);
                println!("  Amount: {}", txn.amount);
                println!("  Payee:  {}", txn.payee_name);
                println!();
                println!("Use --force to confirm deletion");
                return Ok(());
            }

            let deleted = service.delete(txn.id)?;
            println!(
                "Deleted transaction: {} ({} {})",
                deleted.id, deleted.date, deleted.payee_name
            );
        }

        TransactionCommands::Clear { id } => {
            let txn = service
                .find(&id)?
                .ok_or_else(|| EnvelopeError::transaction_not_found(&id))?;

            let cleared = service.clear(txn.id)?;
            println!(
                "Cleared transaction: {} ({})",
                cleared.id, cleared.payee_name
            );
        }

        TransactionCommands::Unclear { id } => {
            let txn = service
                .find(&id)?
                .ok_or_else(|| EnvelopeError::transaction_not_found(&id))?;

            let uncleared = service.unclear(txn.id)?;
            println!(
                "Uncleared transaction: {} ({})",
                uncleared.id, uncleared.payee_name
            );
        }

        TransactionCommands::Unlock { id } => {
            let txn = service
                .find(&id)?
                .ok_or_else(|| EnvelopeError::transaction_not_found(&id))?;

            let unlocked = service.unlock(txn.id)?;
            println!(
                "Unlocked transaction: {} ({}) - now marked as Cleared",
                unlocked.id, unlocked.payee_name
            );
            println!("WARNING: This transaction was previously reconciled.");
            println!("         Editing it may cause discrepancies with your bank statement.");
        }
    }

    Ok(())
}
