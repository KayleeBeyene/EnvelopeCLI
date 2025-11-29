//! CLI command handler for CSV import
//!
//! Handles importing transactions from CSV files with automatic
//! column mapping detection and duplicate checking.

use std::path::Path;

use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{Account, AccountId};
use crate::services::{
    AccountService, ImportPreviewEntry, ImportService, ImportStatus, ParsedTransaction,
};
use crate::storage::Storage;

/// Handle the import command
pub fn handle_import_command(storage: &Storage, file: &str, account: &str) -> EnvelopeResult<()> {
    let account_service = AccountService::new(storage);
    let import_service = ImportService::new(storage);

    let (parsed, target_account) =
        read_and_parse_csv(&import_service, &account_service, file, account)?;

    if parsed.is_empty() {
        println!("No transactions found in CSV file.");
        return Ok(());
    }

    let preview = generate_and_display_preview(&import_service, &parsed, &target_account)?;

    let new_count = preview
        .iter()
        .filter(|e| e.status == ImportStatus::New)
        .count();

    if new_count > 0 {
        execute_import(&import_service, &preview, target_account.id)?;
    }

    Ok(())
}

/// Read and parse CSV file, returning parsed transactions and target account
fn read_and_parse_csv(
    import_service: &ImportService,
    account_service: &AccountService,
    file: &str,
    account: &str,
) -> EnvelopeResult<(Vec<Result<ParsedTransaction, String>>, Account)> {
    let target_account = account_service
        .find(account)?
        .ok_or_else(|| EnvelopeError::account_not_found(account))?;

    let path = Path::new(file);
    if !path.exists() {
        return Err(EnvelopeError::Import(format!("File not found: {}", file)));
    }

    // First, peek at the file to detect the format
    let mut reader = csv::Reader::from_path(path)
        .map_err(|e| EnvelopeError::Import(format!("Failed to open CSV file: {}", e)))?;
    let headers = reader
        .headers()
        .map_err(|e| EnvelopeError::Import(format!("Failed to read CSV headers: {}", e)))?
        .clone();
    let mapping = import_service.detect_mapping_from_headers(&headers);

    // If no header detected, re-read without treating first row as header
    let parsed = if !mapping.has_header {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)
            .map_err(|e| EnvelopeError::Import(format!("Failed to open CSV file: {}", e)))?;
        import_service.parse_csv_from_reader(&mut reader, &mapping)?
    } else {
        import_service.parse_csv_from_reader(&mut reader, &mapping)?
    };

    Ok((parsed, target_account))
}

/// Generate import preview and display summary to user
fn generate_and_display_preview(
    import_service: &ImportService,
    parsed: &[Result<ParsedTransaction, String>],
    target_account: &Account,
) -> EnvelopeResult<Vec<ImportPreviewEntry>> {
    let preview = import_service.generate_preview(parsed, target_account.id)?;

    let new_count = preview
        .iter()
        .filter(|e| e.status == ImportStatus::New)
        .count();
    let dup_count = preview
        .iter()
        .filter(|e| e.status == ImportStatus::Duplicate)
        .count();
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
    } else {
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
    }

    Ok(preview)
}

/// Execute the import and display results
fn execute_import(
    import_service: &ImportService,
    preview: &[ImportPreviewEntry],
    account_id: AccountId,
) -> EnvelopeResult<()> {
    let result = import_service.import_from_preview(
        preview, account_id, None,  // No default category
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
