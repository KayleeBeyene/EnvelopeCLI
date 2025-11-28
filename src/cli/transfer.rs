//! CLI command handler for account transfers
//!
//! Handles transferring funds between accounts, creating linked
//! transaction pairs that maintain balance consistency.

use chrono::NaiveDate;

use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::Money;
use crate::services::{AccountService, TransferService};
use crate::storage::Storage;

/// Handle the transfer command
pub fn handle_transfer_command(
    storage: &Storage,
    from: &str,
    to: &str,
    amount: &str,
    date: Option<&str>,
    memo: Option<String>,
) -> EnvelopeResult<()> {
    let account_service = AccountService::new(storage);
    let transfer_service = TransferService::new(storage);

    // Find source account
    let from_account = account_service.find(from)?.ok_or_else(|| {
        EnvelopeError::account_not_found(from)
    })?;

    // Find destination account
    let to_account = account_service.find(to)?.ok_or_else(|| {
        EnvelopeError::account_not_found(to)
    })?;

    // Parse amount
    let amount = Money::parse(amount).map_err(|e| {
        EnvelopeError::Validation(format!(
            "Invalid amount format: '{}'. Use format like '100.00' or '100'. Error: {}",
            amount, e
        ))
    })?;

    // Parse date (default to today)
    let date = if let Some(date_str) = date {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            EnvelopeError::Validation(format!(
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

    Ok(())
}
