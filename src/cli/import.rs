//! CLI command handler for CSV import
//!
//! Handles importing transactions from CSV files with automatic
//! column mapping detection and duplicate checking.

use std::path::Path;

use crate::error::{EnvelopeError, EnvelopeResult};
use crate::services::{AccountService, ImportService, ImportStatus};
use crate::storage::Storage;

/// Handle the import command
pub fn handle_import_command(
    storage: &Storage,
    file: &str,
    account: &str,
) -> EnvelopeResult<()> {
    let account_service = AccountService::new(storage);
    let import_service = ImportService::new(storage);

    // Find account
    let target_account = account_service.find(account)?.ok_or_else(|| {
        EnvelopeError::account_not_found(account)
    })?;

    let path = Path::new(file);
    if !path.exists() {
        return Err(EnvelopeError::Import(format!("File not found: {}", file)));
    }

    // Try to detect mapping from CSV header
    let content = std::fs::read_to_string(path)
        .map_err(|e| EnvelopeError::Import(format!("Failed to read file: {}", e)))?;
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
    let err_count = preview
        .iter()
        .filter(|e| matches!(e.status, ImportStatus::Error(_)))
        .count();

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
    for entry in preview
        .iter()
        .filter(|e| e.status == ImportStatus::New)
        .take(5)
    {
        println!(
            "  {} {} {}",
            entry.transaction.date, entry.transaction.payee, entry.transaction.amount
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
        None,  // No default category
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

    Ok(())
}
