//! CLI commands for data export
//!
//! Provides commands for exporting data in various formats.

use crate::error::EnvelopeResult;
use crate::export::{csv, json, yaml};
use crate::storage::Storage;
use clap::{Subcommand, ValueEnum};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

/// Export format options
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportFormat {
    /// CSV format (transactions only)
    Csv,
    /// JSON format (full database)
    Json,
    /// YAML format (full database, human-readable)
    Yaml,
}

/// Export subcommands
#[derive(Subcommand, Debug)]
pub enum ExportCommands {
    /// Export all data to a file
    All {
        /// Output file path
        output: PathBuf,

        /// Export format
        #[arg(short, long, value_enum, default_value = "json")]
        format: ExportFormat,

        /// Pretty-print JSON output
        #[arg(long)]
        pretty: bool,
    },

    /// Export transactions to CSV
    Transactions {
        /// Output file path
        output: PathBuf,
    },

    /// Export budget allocations to CSV
    Allocations {
        /// Output file path
        output: PathBuf,

        /// Number of months to export (default: 12)
        #[arg(short, long, default_value = "12")]
        months: usize,
    },

    /// Export accounts to CSV
    Accounts {
        /// Output file path
        output: PathBuf,
    },

    /// Show export information without writing files
    Info,
}

/// Handle export commands
pub fn handle_export_command(storage: &Storage, cmd: ExportCommands) -> EnvelopeResult<()> {
    match cmd {
        ExportCommands::All {
            output,
            format,
            pretty,
        } => handle_export_all(storage, output, format, pretty),
        ExportCommands::Transactions { output } => handle_export_transactions(storage, output),
        ExportCommands::Allocations { output, months } => {
            handle_export_allocations(storage, output, months)
        }
        ExportCommands::Accounts { output } => handle_export_accounts(storage, output),
        ExportCommands::Info => handle_export_info(storage),
    }
}

/// Handle full export
fn handle_export_all(
    storage: &Storage,
    output: PathBuf,
    format: ExportFormat,
    pretty: bool,
) -> EnvelopeResult<()> {
    let file = File::create(&output).map_err(|e| {
        crate::error::EnvelopeError::Export(format!(
            "Failed to create file {}: {}",
            output.display(),
            e
        ))
    })?;
    let mut writer = BufWriter::new(file);

    match format {
        ExportFormat::Csv => {
            // For CSV, export transactions as the primary data
            csv::export_transactions_csv(storage, &mut writer)?;
            println!("Transactions exported to: {}", output.display());
            println!("Note: CSV format exports transactions only. Use JSON or YAML for full database export.");
        }
        ExportFormat::Json => {
            json::export_full_json(storage, &mut writer, pretty)?;
            println!("Full database exported to: {}", output.display());
        }
        ExportFormat::Yaml => {
            yaml::export_full_yaml(storage, &mut writer)?;
            println!("Full database exported to: {}", output.display());
        }
    }

    Ok(())
}

/// Handle transactions export
fn handle_export_transactions(storage: &Storage, output: PathBuf) -> EnvelopeResult<()> {
    let file = File::create(&output).map_err(|e| {
        crate::error::EnvelopeError::Export(format!(
            "Failed to create file {}: {}",
            output.display(),
            e
        ))
    })?;
    let mut writer = BufWriter::new(file);

    csv::export_transactions_csv(storage, &mut writer)?;

    let count = storage.transactions.get_all()?.len();
    println!("Exported {} transactions to: {}", count, output.display());

    Ok(())
}

/// Handle allocations export
fn handle_export_allocations(
    storage: &Storage,
    output: PathBuf,
    months: usize,
) -> EnvelopeResult<()> {
    use crate::models::BudgetPeriod;

    let file = File::create(&output).map_err(|e| {
        crate::error::EnvelopeError::Export(format!(
            "Failed to create file {}: {}",
            output.display(),
            e
        ))
    })?;
    let mut writer = BufWriter::new(file);

    // Generate periods
    let current = BudgetPeriod::current_month();
    let periods: Vec<_> = (0..months)
        .map(|i| {
            let mut p = current.clone();
            for _ in 0..i {
                p = p.prev();
            }
            p
        })
        .collect();

    csv::export_allocations_csv(storage, &mut writer, Some(periods))?;

    println!(
        "Exported {} months of budget allocations to: {}",
        months,
        output.display()
    );

    Ok(())
}

/// Handle accounts export
fn handle_export_accounts(storage: &Storage, output: PathBuf) -> EnvelopeResult<()> {
    let file = File::create(&output).map_err(|e| {
        crate::error::EnvelopeError::Export(format!(
            "Failed to create file {}: {}",
            output.display(),
            e
        ))
    })?;
    let mut writer = BufWriter::new(file);

    csv::export_accounts_csv(storage, &mut writer)?;

    let count = storage.accounts.get_all()?.len();
    println!("Exported {} accounts to: {}", count, output.display());

    Ok(())
}

/// Show export information
fn handle_export_info(storage: &Storage) -> EnvelopeResult<()> {
    let export = json::FullExport::from_storage(storage)?;

    println!("Export Information");
    println!("==================\n");

    println!("Schema Version: {}", export.schema_version);
    println!("App Version:    {}", export.app_version);
    println!();

    println!("Data Summary:");
    println!("  Accounts:      {}", export.metadata.account_count);
    println!("  Transactions:  {}", export.metadata.transaction_count);
    println!("  Categories:    {}", export.metadata.category_count);
    println!("  Allocations:   {}", export.metadata.allocation_count);
    println!("  Payees:        {}", export.metadata.payee_count);
    println!();

    if let Some(earliest) = &export.metadata.earliest_transaction {
        println!("Transaction Date Range:");
        println!("  Earliest: {}", earliest);
    }
    if let Some(latest) = &export.metadata.latest_transaction {
        println!("  Latest:   {}", latest);
    }

    println!("\nAvailable Export Formats:");
    println!("  csv  - CSV format (transactions, allocations, or accounts)");
    println!("  json - JSON format (full database, machine-readable)");
    println!("  yaml - YAML format (full database, human-readable)");

    println!("\nExamples:");
    println!("  envelope export all backup.json --format json --pretty");
    println!("  envelope export transactions txns.csv");
    println!("  envelope export accounts accounts.csv");

    Ok(())
}
