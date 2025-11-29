//! Income CLI commands
//!
//! Implements CLI commands for managing expected income per budget period.

use clap::Subcommand;

use crate::config::settings::Settings;
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::Money;
use crate::services::{BudgetService, IncomeService, PeriodService};
use crate::storage::Storage;

/// Income subcommands
#[derive(Subcommand)]
pub enum IncomeCommands {
    /// Set expected income for a period
    Set {
        /// Expected income amount (e.g., "5000" or "5000.00")
        amount: String,

        /// Budget period (e.g., "2025-01" for January 2025)
        #[arg(short, long)]
        period: Option<String>,

        /// Notes about this income expectation
        #[arg(short, long)]
        notes: Option<String>,
    },

    /// Show expected income for a period
    Show {
        /// Budget period (defaults to current month)
        #[arg(short, long)]
        period: Option<String>,
    },

    /// Remove expected income for a period
    Remove {
        /// Budget period
        #[arg(short, long)]
        period: Option<String>,
    },

    /// Compare expected income vs budgeted amounts
    Compare {
        /// Budget period (defaults to current month)
        #[arg(short, long)]
        period: Option<String>,
    },
}

/// Handle an income command
pub fn handle_income_command(
    storage: &Storage,
    settings: &Settings,
    cmd: IncomeCommands,
) -> EnvelopeResult<()> {
    let period_service = PeriodService::new(settings);
    let income_service = IncomeService::new(storage);
    let budget_service = BudgetService::new(storage);

    match cmd {
        IncomeCommands::Set {
            amount,
            period,
            notes,
        } => {
            let period = period_service.parse_or_current(period.as_deref())?;
            let amount = Money::parse(&amount)
                .map_err(|e| EnvelopeError::Validation(format!("Invalid amount: {}", e)))?;
            let friendly = period_service.format_period_friendly(&period);

            let expectation = income_service.set_expected_income(&period, amount, notes)?;

            println!(
                "Set expected income for {} to {}",
                friendly, expectation.expected_amount
            );

            // Show comparison if budget exists
            if let Some(overage) = budget_service.is_over_expected_income(&period)? {
                println!(
                    "Warning: You're budgeting {} more than expected income!",
                    overage
                );
            } else if let Some(remaining) =
                budget_service.get_remaining_to_budget_from_income(&period)?
            {
                if remaining.is_positive() {
                    println!("Remaining to budget from income: {}", remaining);
                } else {
                    println!("Budget matches expected income.");
                }
            }
        }

        IncomeCommands::Show { period } => {
            let period = period_service.parse_or_current(period.as_deref())?;
            let friendly = period_service.format_period_friendly(&period);

            if let Some(expectation) = income_service.get_income_expectation(&period) {
                println!("Expected Income for {}", friendly);
                println!("{}", "-".repeat(40));
                println!("Amount: {}", expectation.expected_amount);
                if !expectation.notes.is_empty() {
                    println!("Notes:  {}", expectation.notes);
                }

                // Show budget comparison
                let overview = budget_service.get_budget_overview(&period)?;
                println!();
                println!("Budget Comparison:");
                println!("  Expected Income:  {}", expectation.expected_amount);
                println!("  Total Budgeted:   {}", overview.total_budgeted);

                let diff = expectation.expected_amount - overview.total_budgeted;
                if diff.is_negative() {
                    println!("  Over Budget:      {} ⚠", diff.abs());
                } else {
                    println!("  Remaining:        {} ✓", diff);
                }
            } else {
                println!("No expected income set for {}", friendly);
                println!("Use 'envelope income set <amount>' to set expected income.");
            }
        }

        IncomeCommands::Remove { period } => {
            let period = period_service.parse_or_current(period.as_deref())?;
            let friendly = period_service.format_period_friendly(&period);

            if income_service.delete_expected_income(&period)? {
                println!("Removed expected income for {}", friendly);
            } else {
                println!("No expected income was set for {}", friendly);
            }
        }

        IncomeCommands::Compare { period } => {
            let period = period_service.parse_or_current(period.as_deref())?;
            let friendly = period_service.format_period_friendly(&period);
            let overview = budget_service.get_budget_overview(&period)?;

            println!("Income vs Budget Comparison for {}", friendly);
            println!("{}", "=".repeat(50));

            if let Some(expectation) = income_service.get_income_expectation(&period) {
                println!("Expected Income:     {:>12}", expectation.expected_amount);
                println!("Total Budgeted:      {:>12}", overview.total_budgeted);
                println!("{}", "-".repeat(50));

                let diff = expectation.expected_amount - overview.total_budgeted;
                if diff.is_negative() {
                    println!("OVER BUDGET:         {:>12} ⚠", diff.abs());
                    println!();
                    println!("Warning: You're budgeting more than you expect to earn!");
                    println!("Consider reducing budget allocations or increasing expected income.");
                } else if diff.is_zero() {
                    println!("Remaining to Budget: {:>12} ✓", diff);
                    println!();
                    println!("Your budget exactly matches expected income.");
                } else {
                    println!("Remaining to Budget: {:>12} ✓", diff);
                    println!();
                    println!("You have {} available to budget.", diff);
                }

                // Show additional info
                if !expectation.notes.is_empty() {
                    println!();
                    println!("Notes: {}", expectation.notes);
                }
            } else {
                println!("Expected Income:     Not set");
                println!("Total Budgeted:      {:>12}", overview.total_budgeted);
                println!();
                println!("Tip: Set expected income with 'envelope income set <amount>'");
            }

            // Show available to budget (from account balances)
            println!();
            println!("{}", "-".repeat(50));
            println!(
                "Available to Budget (from accounts): {:>12}",
                overview.available_to_budget
            );
        }
    }

    Ok(())
}
