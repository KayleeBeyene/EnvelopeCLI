//! CLI commands for reports
//!
//! Provides commands for generating and exporting various financial reports.

use crate::error::EnvelopeResult;
use crate::models::BudgetPeriod;
use crate::reports::{
    AccountRegisterReport, BudgetOverviewReport, NetWorthReport, RegisterFilter, SpendingReport,
};
use crate::services::AccountService;
use crate::storage::Storage;
use chrono::NaiveDate;
use clap::Subcommand;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

/// Report subcommands
#[derive(Subcommand, Debug)]
pub enum ReportCommands {
    /// Generate a budget overview report
    #[command(alias = "budget-overview")]
    Budget {
        /// Budget period (e.g., "2025-01" for January 2025)
        #[arg(short, long)]
        period: Option<String>,

        /// Export to CSV file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate a spending report by category
    Spending {
        /// Start date (YYYY-MM-DD)
        #[arg(short, long)]
        start: Option<String>,

        /// End date (YYYY-MM-DD)
        #[arg(short, long)]
        end: Option<String>,

        /// Budget period to report on (alternative to start/end)
        #[arg(short, long)]
        period: Option<String>,

        /// Export to CSV file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show top N categories only
        #[arg(long)]
        top: Option<usize>,
    },

    /// Generate an account register report
    #[command(alias = "transactions")]
    Register {
        /// Account name or ID
        account: String,

        /// Start date (YYYY-MM-DD)
        #[arg(short, long)]
        start: Option<String>,

        /// End date (YYYY-MM-DD)
        #[arg(short, long)]
        end: Option<String>,

        /// Filter by payee (partial match)
        #[arg(long)]
        payee: Option<String>,

        /// Show only uncategorized transactions
        #[arg(long)]
        uncategorized: bool,

        /// Export to CSV file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate a net worth report
    #[command(alias = "networth")]
    NetWorth {
        /// Include archived accounts
        #[arg(short, long)]
        all: bool,

        /// Export to CSV file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// Handle report commands
pub fn handle_report_command(storage: &Storage, cmd: ReportCommands) -> EnvelopeResult<()> {
    match cmd {
        ReportCommands::Budget { period, output } => handle_budget_report(storage, period, output),
        ReportCommands::Spending {
            start,
            end,
            period,
            output,
            top,
        } => handle_spending_report(storage, start, end, period, output, top),
        ReportCommands::Register {
            account,
            start,
            end,
            payee,
            uncategorized,
            output,
        } => handle_register_report(storage, account, start, end, payee, uncategorized, output),
        ReportCommands::NetWorth { all, output } => handle_net_worth_report(storage, all, output),
    }
}

/// Handle budget overview report
fn handle_budget_report(
    storage: &Storage,
    period: Option<String>,
    output: Option<PathBuf>,
) -> EnvelopeResult<()> {
    // Parse period or use current
    let budget_period = if let Some(period_str) = period {
        BudgetPeriod::parse(&period_str).map_err(|e| {
            crate::error::EnvelopeError::Validation(format!(
                "Invalid period format: {}. Use YYYY-MM (e.g., 2025-01)",
                e
            ))
        })?
    } else {
        BudgetPeriod::current_month()
    };

    // Generate report
    let report = BudgetOverviewReport::generate(storage, &budget_period)?;

    // Output
    if let Some(path) = output {
        let file = File::create(&path).map_err(|e| {
            crate::error::EnvelopeError::Export(format!(
                "Failed to create file {}: {}",
                path.display(),
                e
            ))
        })?;
        let mut writer = BufWriter::new(file);
        report.export_csv(&mut writer)?;
        println!("Budget report exported to: {}", path.display());
    } else {
        println!("{}", report.format_terminal());
    }

    Ok(())
}

/// Handle spending report
fn handle_spending_report(
    storage: &Storage,
    start: Option<String>,
    end: Option<String>,
    period: Option<String>,
    output: Option<PathBuf>,
    top: Option<usize>,
) -> EnvelopeResult<()> {
    // Determine date range
    let (start_date, end_date) = if let Some(period_str) = period {
        let budget_period = BudgetPeriod::parse(&period_str).map_err(|e| {
            crate::error::EnvelopeError::Validation(format!(
                "Invalid period format: {}. Use YYYY-MM (e.g., 2025-01)",
                e
            ))
        })?;
        (budget_period.start_date(), budget_period.end_date())
    } else {
        let start_date = if let Some(s) = start {
            NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(|_| {
                crate::error::EnvelopeError::Validation(format!(
                    "Invalid start date format: {}. Use YYYY-MM-DD",
                    s
                ))
            })?
        } else {
            // Default to start of current month
            let today = chrono::Local::now().date_naive();
            NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today)
        };

        let end_date = if let Some(e) = end {
            NaiveDate::parse_from_str(&e, "%Y-%m-%d").map_err(|_| {
                crate::error::EnvelopeError::Validation(format!(
                    "Invalid end date format: {}. Use YYYY-MM-DD",
                    e
                ))
            })?
        } else {
            // Default to today
            chrono::Local::now().date_naive()
        };

        (start_date, end_date)
    };

    // Generate report
    let report = SpendingReport::generate(storage, start_date, end_date)?;

    // Output
    if let Some(path) = output {
        let file = File::create(&path).map_err(|e| {
            crate::error::EnvelopeError::Export(format!(
                "Failed to create file {}: {}",
                path.display(),
                e
            ))
        })?;
        let mut writer = BufWriter::new(file);
        report.export_csv(&mut writer)?;
        println!("Spending report exported to: {}", path.display());
    } else if let Some(n) = top {
        // Show top N categories only
        println!(
            "Top {} Spending Categories: {} to {}\n",
            n, start_date, end_date
        );
        println!("{:<35} {:>12} {:>8}", "Category", "Amount", "%");
        println!("{}", "-".repeat(60));

        for cat in report.top_categories(n) {
            println!(
                "{:<35} {:>12} {:>7.1}%",
                cat.category_name,
                cat.total_spending.abs(),
                cat.percentage
            );
        }
        println!("\nTotal Spending: {}", report.total_spending.abs());
    } else {
        println!("{}", report.format_terminal());
    }

    Ok(())
}

/// Handle account register report
fn handle_register_report(
    storage: &Storage,
    account: String,
    start: Option<String>,
    end: Option<String>,
    payee: Option<String>,
    uncategorized: bool,
    output: Option<PathBuf>,
) -> EnvelopeResult<()> {
    let account_service = AccountService::new(storage);

    // Find account
    let account = account_service
        .find(&account)?
        .ok_or_else(|| crate::error::EnvelopeError::account_not_found(&account))?;

    // Build filter
    let filter = RegisterFilter {
        start_date: start
            .map(|s| {
                NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(|_| {
                    crate::error::EnvelopeError::Validation(format!(
                        "Invalid start date format: {}. Use YYYY-MM-DD",
                        s
                    ))
                })
            })
            .transpose()?,
        end_date: end
            .map(|s| {
                NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(|_| {
                    crate::error::EnvelopeError::Validation(format!(
                        "Invalid end date format: {}. Use YYYY-MM-DD",
                        s
                    ))
                })
            })
            .transpose()?,
        payee_contains: payee,
        uncategorized_only: uncategorized,
        ..Default::default()
    };

    // Generate report
    let report = AccountRegisterReport::generate(storage, account.id, filter)?;

    // Output
    if let Some(path) = output {
        let file = File::create(&path).map_err(|e| {
            crate::error::EnvelopeError::Export(format!(
                "Failed to create file {}: {}",
                path.display(),
                e
            ))
        })?;
        let mut writer = BufWriter::new(file);
        report.export_csv(&mut writer)?;
        println!("Register report exported to: {}", path.display());
    } else {
        println!("{}", report.format_terminal());
    }

    Ok(())
}

/// Handle net worth report
fn handle_net_worth_report(
    storage: &Storage,
    include_archived: bool,
    output: Option<PathBuf>,
) -> EnvelopeResult<()> {
    // Generate report
    let report = NetWorthReport::generate(storage, include_archived)?;

    // Output
    if let Some(path) = output {
        let file = File::create(&path).map_err(|e| {
            crate::error::EnvelopeError::Export(format!(
                "Failed to create file {}: {}",
                path.display(),
                e
            ))
        })?;
        let mut writer = BufWriter::new(file);
        report.export_csv(&mut writer)?;
        println!("Net worth report exported to: {}", path.display());
    } else {
        println!("{}", report.format_terminal());
    }

    Ok(())
}

use chrono::Datelike;
