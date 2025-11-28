//! Budget target CLI commands
//!
//! Implements CLI commands for managing recurring budget targets on categories.

use chrono::NaiveDate;
use clap::Subcommand;

use crate::config::settings::Settings;
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{Money, TargetCadence};
use crate::services::{BudgetService, CategoryService, PeriodService};
use crate::storage::Storage;

/// Target subcommands
#[derive(Subcommand)]
pub enum TargetCommands {
    /// Set a budget target for a category
    Set {
        /// Category name or ID
        category: String,
        /// Target amount (e.g., "500" or "500.00")
        amount: String,
        /// Target cadence: weekly, monthly, yearly, custom, or by-date
        #[arg(short, long, default_value = "monthly")]
        cadence: String,
        /// Number of days for custom cadence (required when cadence is "custom")
        #[arg(long)]
        days: Option<u32>,
        /// Target date for by-date cadence (YYYY-MM-DD, required when cadence is "by-date")
        #[arg(long)]
        date: Option<String>,
    },

    /// List all active budget targets
    List,

    /// Show the target for a specific category
    Show {
        /// Category name or ID
        category: String,
    },

    /// Delete the target for a category
    Delete {
        /// Category name or ID
        category: String,
    },

    /// Auto-fill budgets from targets for a period
    #[command(name = "auto-fill")]
    AutoFill {
        /// Budget period (e.g., "2025-01", "current")
        #[arg(short, long)]
        period: Option<String>,
    },
}

/// Handle a target command
pub fn handle_target_command(
    storage: &Storage,
    settings: &Settings,
    cmd: TargetCommands,
) -> EnvelopeResult<()> {
    let period_service = PeriodService::new(settings);

    match cmd {
        TargetCommands::Set {
            category,
            amount,
            cadence,
            days,
            date,
        } => {
            let category_service = CategoryService::new(storage);
            let cat = category_service
                .find_category(&category)?
                .ok_or_else(|| EnvelopeError::category_not_found(&category))?;

            let amount = Money::parse(&amount)
                .map_err(|e| EnvelopeError::Validation(format!("Invalid amount: {}", e)))?;

            let cadence = parse_cadence(&cadence, days, date.as_deref())?;

            let budget_service = BudgetService::new(storage);
            let target = budget_service.set_target(cat.id, amount, cadence.clone())?;

            println!(
                "Set target for '{}': {} {}",
                cat.name, target.amount, cadence
            );

            // Show what the suggested amount would be for the current period
            let current_period = period_service.current_period();
            let suggested = target.calculate_for_period(&current_period);
            println!(
                "  Suggested for {}: {}",
                period_service.format_period_friendly(&current_period),
                suggested
            );
        }

        TargetCommands::List => {
            let budget_service = BudgetService::new(storage);
            let category_service = CategoryService::new(storage);
            let targets = budget_service.get_all_targets()?;

            if targets.is_empty() {
                println!("No budget targets set.");
                println!();
                println!("Use 'envelope target set <category> <amount>' to create a target.");
            } else {
                println!("Budget Targets:");
                println!("{}", "-".repeat(60));
                println!("{:25} {:>12} {:>15}", "Category", "Amount", "Cadence");
                println!("{}", "-".repeat(60));

                let current_period = period_service.current_period();

                for target in &targets {
                    let cat_name = category_service
                        .get_category(target.category_id)?
                        .map(|c| c.name)
                        .unwrap_or_else(|| "Unknown".to_string());

                    let suggested = target.calculate_for_period(&current_period);

                    println!(
                        "{:25} {:>12} {:>15}",
                        cat_name, target.amount, target.cadence
                    );

                    // If the cadence results in a different amount for the current period, show it
                    if suggested != target.amount {
                        println!(
                            "{:25} {:>12} (for {})",
                            "",
                            suggested,
                            period_service.format_period_friendly(&current_period)
                        );
                    }
                }

                println!("{}", "-".repeat(60));
                println!("{} target(s) total", targets.len());
            }
        }

        TargetCommands::Show { category } => {
            let category_service = CategoryService::new(storage);
            let cat = category_service
                .find_category(&category)?
                .ok_or_else(|| EnvelopeError::category_not_found(&category))?;

            let budget_service = BudgetService::new(storage);
            let target = budget_service.get_target(cat.id)?;

            match target {
                Some(t) => {
                    println!("Target for '{}':", cat.name);
                    println!("  Amount:  {}", t.amount);
                    println!("  Cadence: {}", t.cadence);
                    println!("  Active:  {}", if t.active { "Yes" } else { "No" });
                    if !t.notes.is_empty() {
                        println!("  Notes:   {}", t.notes);
                    }
                    println!("  Created: {}", t.created_at.format("%Y-%m-%d %H:%M"));

                    // Show suggested amounts for a few periods
                    println!();
                    println!("Suggested amounts:");
                    let current = period_service.current_period();
                    for i in 0..3 {
                        let period = if i == 0 {
                            current.clone()
                        } else {
                            let mut p = current.clone();
                            for _ in 0..i {
                                p = p.next();
                            }
                            p
                        };
                        let suggested = t.calculate_for_period(&period);
                        let label = if i == 0 { " (current)" } else { "" };
                        println!(
                            "  {}{}: {}",
                            period_service.format_period_friendly(&period),
                            label,
                            suggested
                        );
                    }
                }
                None => {
                    println!("No target set for '{}'.", cat.name);
                    println!();
                    println!(
                        "Use 'envelope target set {} <amount>' to create one.",
                        cat.name
                    );
                }
            }
        }

        TargetCommands::Delete { category } => {
            let category_service = CategoryService::new(storage);
            let cat = category_service
                .find_category(&category)?
                .ok_or_else(|| EnvelopeError::category_not_found(&category))?;

            let budget_service = BudgetService::new(storage);
            let deleted = budget_service.remove_target(cat.id)?;

            if deleted {
                println!("Deleted target for '{}'.", cat.name);
            } else {
                println!("No target found for '{}'.", cat.name);
            }
        }

        TargetCommands::AutoFill { period } => {
            let period = period_service.parse_or_current(period.as_deref())?;
            let friendly = period_service.format_period_friendly(&period);

            let budget_service = BudgetService::new(storage);
            let category_service = CategoryService::new(storage);
            let allocations = budget_service.auto_fill_all_targets(&period)?;

            if allocations.is_empty() {
                println!("No targets to auto-fill for {}.", friendly);
                println!();
                println!("Use 'envelope target set <category> <amount>' to create targets first.");
            } else {
                println!("Auto-filled budgets from targets for {}:", friendly);
                println!();

                for allocation in &allocations {
                    let cat_name = category_service
                        .get_category(allocation.category_id)?
                        .map(|c| c.name)
                        .unwrap_or_else(|| "Unknown".to_string());

                    println!("  {}: {}", cat_name, allocation.budgeted);
                }

                println!();
                println!("{} category/categories updated.", allocations.len());

                // Show Available to Budget
                let atb = budget_service.get_available_to_budget(&period)?;
                if atb.is_negative() {
                    println!();
                    println!(
                        "⚠️  Warning: Overbudgeted by {}. Available to Budget: {}",
                        atb.abs(),
                        atb
                    );
                } else if atb.is_positive() {
                    println!();
                    println!("Available to Budget: {}", atb);
                }
            }
        }
    }

    Ok(())
}

/// Parse the cadence string and optional parameters into a TargetCadence
fn parse_cadence(
    cadence: &str,
    days: Option<u32>,
    date: Option<&str>,
) -> EnvelopeResult<TargetCadence> {
    match cadence.to_lowercase().as_str() {
        "weekly" => Ok(TargetCadence::Weekly),
        "monthly" => Ok(TargetCadence::Monthly),
        "yearly" | "annual" | "annually" => Ok(TargetCadence::Yearly),
        "custom" => {
            let days = days.ok_or_else(|| {
                EnvelopeError::Validation(
                    "Custom cadence requires --days parameter (e.g., --days 14)".to_string(),
                )
            })?;
            if days == 0 {
                return Err(EnvelopeError::Validation(
                    "Custom interval must be at least 1 day".to_string(),
                ));
            }
            Ok(TargetCadence::Custom { days })
        }
        "by-date" | "bydate" | "by_date" => {
            let date_str = date.ok_or_else(|| {
                EnvelopeError::Validation(
                    "By-date cadence requires --date parameter (e.g., --date 2025-12-25)"
                        .to_string(),
                )
            })?;
            let target_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e| {
                EnvelopeError::Validation(format!(
                    "Invalid date format '{}'. Use YYYY-MM-DD: {}",
                    date_str, e
                ))
            })?;
            Ok(TargetCadence::ByDate { target_date })
        }
        _ => Err(EnvelopeError::Validation(format!(
            "Unknown cadence '{}'. Valid options: weekly, monthly, yearly, custom, by-date",
            cadence
        ))),
    }
}
