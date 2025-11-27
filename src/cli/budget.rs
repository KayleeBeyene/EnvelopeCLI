//! Budget CLI commands
//!
//! Implements CLI commands for budget management including period navigation,
//! allocation, and overview.

use clap::Subcommand;

use crate::config::settings::Settings;
use crate::error::EnvelopeResult;
use crate::services::{BudgetService, CategoryService, PeriodService};
use crate::storage::Storage;

/// Budget subcommands
#[derive(Subcommand)]
pub enum BudgetCommands {
    /// Show budget overview for a period
    Overview {
        /// Budget period (e.g., "2025-01", "January", "current", "last")
        #[arg(short, long)]
        period: Option<String>,
    },

    /// Show information about the current period
    Period {
        /// Budget period (e.g., "2025-01", "January", "current", "last")
        period: Option<String>,
    },

    /// List recent budget periods
    Periods {
        /// Number of periods to show
        #[arg(short, long, default_value = "6")]
        count: usize,
    },

    /// Go to the previous period
    Prev,

    /// Go to the next period
    Next,

    // These will be implemented in Step 9:
    /// Assign funds to a category
    Assign {
        /// Category name
        category: String,
        /// Amount (e.g., "100" or "100.00")
        amount: String,
        /// Budget period
        #[arg(short, long)]
        period: Option<String>,
    },

    /// Move funds between categories
    Move {
        /// Source category
        from: String,
        /// Destination category
        to: String,
        /// Amount
        amount: String,
        /// Budget period
        #[arg(short, long)]
        period: Option<String>,
    },

    /// Apply rollover from previous period
    Rollover {
        /// Budget period to apply rollover to (defaults to current)
        #[arg(short, long)]
        period: Option<String>,
    },

    /// Show overspent categories
    Overspent {
        /// Budget period
        #[arg(short, long)]
        period: Option<String>,
    },
}

/// Handle a budget command
pub fn handle_budget_command(
    storage: &Storage,
    settings: &Settings,
    cmd: BudgetCommands,
) -> EnvelopeResult<()> {
    let period_service = PeriodService::new(settings);

    match cmd {
        BudgetCommands::Overview { period } => {
            let period = period_service.parse_or_current(period.as_deref())?;
            let friendly = period_service.format_period_friendly(&period);

            println!("Budget Overview: {}", friendly);
            println!("{}", "=".repeat(72));

            // Get categories with groups
            let category_service = CategoryService::new(storage);
            let groups = category_service.list_groups_with_categories()?;

            // Get budget service for summaries
            let budget_service = BudgetService::new(storage);

            // Calculate totals
            let mut total_budgeted = crate::models::Money::zero();
            let mut total_carryover = crate::models::Money::zero();
            let mut total_activity = crate::models::Money::zero();
            let mut total_available = crate::models::Money::zero();
            let mut has_any_carryover = false;

            // First pass: check if any categories have carryover
            for gwc in &groups {
                for category in &gwc.categories {
                    let summary = budget_service.get_category_summary(category.id, &period)?;
                    if !summary.carryover.is_zero() {
                        has_any_carryover = true;
                        break;
                    }
                }
                if has_any_carryover {
                    break;
                }
            }

            for gwc in &groups {
                if gwc.categories.is_empty() {
                    continue;
                }

                println!("\n{}", gwc.group.name);
                if has_any_carryover {
                    println!(
                        "{:26} {:>10} {:>10} {:>10} {:>10}",
                        "", "Budgeted", "Carryover", "Activity", "Available"
                    );
                } else {
                    println!(
                        "{:30} {:>10} {:>10} {:>10}",
                        "", "Budgeted", "Activity", "Available"
                    );
                }
                println!("{}", "-".repeat(72));

                for category in &gwc.categories {
                    let summary = budget_service.get_category_summary(category.id, &period)?;

                    total_budgeted += summary.budgeted;
                    total_carryover += summary.carryover;
                    total_activity += summary.activity;
                    total_available += summary.available;

                    let status = if summary.is_overspent() {
                        "⚠"
                    } else if let Some(goal) = category.goal_amount {
                        let goal_money = crate::models::Money::from_cents(goal);
                        if summary.budgeted >= goal_money {
                            "✓"
                        } else {
                            "✗"
                        }
                    } else {
                        ""
                    };

                    if has_any_carryover {
                        println!(
                            "  {:24} {:>10} {:>10} {:>10} {:>10} {}",
                            category.name,
                            summary.budgeted,
                            summary.carryover,
                            summary.activity,
                            summary.available,
                            status
                        );
                    } else {
                        println!(
                            "  {:28} {:>10} {:>10} {:>10} {}",
                            category.name,
                            summary.budgeted,
                            summary.activity,
                            summary.available,
                            status
                        );
                    }
                }
            }

            println!("\n{}", "=".repeat(72));
            if has_any_carryover {
                println!(
                    "{:26} {:>10} {:>10} {:>10} {:>10}",
                    "TOTALS:",
                    total_budgeted,
                    total_carryover,
                    total_activity,
                    total_available
                );
            } else {
                println!(
                    "{:30} {:>10} {:>10} {:>10}",
                    "TOTALS:",
                    total_budgeted,
                    total_activity,
                    total_available
                );
            }

            // Show available to budget
            let available_to_budget = budget_service.get_available_to_budget(&period)?;
            println!("\n{:30} {:>10}", "Available to Budget:", available_to_budget);

            if available_to_budget.is_negative() {
                println!("\n⚠️  Warning: Overbudgeted by {}", available_to_budget.abs());
            } else if available_to_budget.is_positive() {
                println!("\n📌 Tip: You have {} ready to assign!", available_to_budget);
            } else {
                println!("\n✅ Budget is balanced!");
            }

            // Show warning about overspent categories
            let overspent = budget_service.get_overspent_categories(&period)?;
            if !overspent.is_empty() {
                println!(
                    "\n⚠️  {} category/categories overspent. Run 'envelope budget overspent' for details.",
                    overspent.len()
                );
            }
        }

        BudgetCommands::Period { period } => {
            let period = period_service.parse_or_current(period.as_deref())?;
            let friendly = period_service.format_period_friendly(&period);
            let is_current = period_service.is_current(&period);

            println!("Budget Period: {}", friendly);
            println!("  Format: {}", period);
            println!("  Start:  {}", period.start_date());
            println!("  End:    {}", period.end_date());
            if is_current {
                println!("  Status: Current period");
            }
        }

        BudgetCommands::Periods { count } => {
            println!("Recent Budget Periods:");
            println!();

            let periods = period_service.recent_periods(count);
            for period in periods {
                let friendly = period_service.format_period_friendly(&period);
                let marker = if period_service.is_current(&period) {
                    " <- current"
                } else {
                    ""
                };
                println!("  {} ({}){}", friendly, period, marker);
            }
        }

        BudgetCommands::Prev => {
            let current = period_service.current_period();
            let prev = period_service.previous_period(&current);
            let friendly = period_service.format_period_friendly(&prev);
            println!("Previous period: {} ({})", friendly, prev);
        }

        BudgetCommands::Next => {
            let current = period_service.current_period();
            let next = period_service.next_period(&current);
            let friendly = period_service.format_period_friendly(&next);
            println!("Next period: {} ({})", friendly, next);
        }

        BudgetCommands::Assign {
            category,
            amount,
            period,
        } => {
            let period = period_service.parse_or_current(period.as_deref())?;
            let amount = crate::models::Money::parse(&amount).map_err(|e| {
                crate::error::EnvelopeError::Validation(format!("Invalid amount: {}", e))
            })?;

            let category_service = CategoryService::new(storage);
            let cat = category_service.find_category(&category)?.ok_or_else(|| {
                crate::error::EnvelopeError::category_not_found(&category)
            })?;

            let budget_service = BudgetService::new(storage);
            let allocation = budget_service.assign_to_category(cat.id, &period, amount)?;

            println!(
                "Assigned {} to '{}' for {}",
                allocation.budgeted,
                cat.name,
                period_service.format_period_friendly(&period)
            );

            // Show updated Available to Budget
            let atb = budget_service.get_available_to_budget(&period)?;
            if atb.is_negative() {
                println!("Warning: Overbudgeted! Available to Budget: {}", atb);
            } else {
                println!("Available to Budget: {}", atb);
            }
        }

        BudgetCommands::Move {
            from,
            to,
            amount,
            period,
        } => {
            let period = period_service.parse_or_current(period.as_deref())?;
            let amount = crate::models::Money::parse(&amount).map_err(|e| {
                crate::error::EnvelopeError::Validation(format!("Invalid amount: {}", e))
            })?;

            let category_service = CategoryService::new(storage);
            let from_cat = category_service.find_category(&from)?.ok_or_else(|| {
                crate::error::EnvelopeError::category_not_found(&from)
            })?;
            let to_cat = category_service.find_category(&to)?.ok_or_else(|| {
                crate::error::EnvelopeError::category_not_found(&to)
            })?;

            let budget_service = BudgetService::new(storage);
            budget_service.move_between_categories(from_cat.id, to_cat.id, &period, amount)?;

            println!(
                "Moved {} from '{}' to '{}' for {}",
                amount,
                from_cat.name,
                to_cat.name,
                period_service.format_period_friendly(&period)
            );
        }

        BudgetCommands::Rollover { period } => {
            let period = period_service.parse_or_current(period.as_deref())?;
            let friendly = period_service.format_period_friendly(&period);
            let prev_period = period.prev();
            let prev_friendly = period_service.format_period_friendly(&prev_period);

            println!("Applying rollover from {} to {}...", prev_friendly, friendly);
            println!();

            let budget_service = BudgetService::new(storage);
            let category_service = CategoryService::new(storage);
            let allocations = budget_service.apply_rollover_all(&period)?;

            let mut positive_count = 0;
            let mut negative_count = 0;
            let mut total_carryover = crate::models::Money::zero();

            for alloc in &allocations {
                if !alloc.carryover.is_zero() {
                    let cat = category_service.get_category(alloc.category_id)?;
                    let cat_name = cat.map(|c| c.name).unwrap_or_else(|| "Unknown".to_string());

                    if alloc.carryover.is_positive() {
                        println!("  {} +{} (surplus)", cat_name, alloc.carryover);
                        positive_count += 1;
                    } else {
                        println!("  {} {} (deficit)", cat_name, alloc.carryover);
                        negative_count += 1;
                    }
                    total_carryover += alloc.carryover;
                }
            }

            println!();
            if positive_count == 0 && negative_count == 0 {
                println!("No carryover to apply (all categories had zero balance).");
            } else {
                println!(
                    "Applied rollover to {} categories ({} surplus, {} deficit)",
                    positive_count + negative_count,
                    positive_count,
                    negative_count
                );
                println!("Net carryover: {}", total_carryover);
            }
        }

        BudgetCommands::Overspent { period } => {
            let period = period_service.parse_or_current(period.as_deref())?;
            let friendly = period_service.format_period_friendly(&period);

            let budget_service = BudgetService::new(storage);
            let category_service = CategoryService::new(storage);
            let overspent = budget_service.get_overspent_categories(&period)?;

            if overspent.is_empty() {
                println!("No overspent categories for {}.", friendly);
                println!("All categories are within budget!");
            } else {
                println!("Overspent Categories for {}:", friendly);
                println!("{}", "-".repeat(50));
                println!("{:30} {:>10} {:>10}", "Category", "Available", "Overspent");
                println!("{}", "-".repeat(50));

                let mut total_overspent = crate::models::Money::zero();

                for summary in &overspent {
                    let cat = category_service.get_category(summary.category_id)?;
                    let cat_name = cat.map(|c| c.name).unwrap_or_else(|| "Unknown".to_string());
                    let overspent_amount = summary.available.abs();

                    println!(
                        "{:30} {:>10} {:>10}",
                        cat_name, summary.available, overspent_amount
                    );

                    total_overspent += overspent_amount;
                }

                println!("{}", "-".repeat(50));
                println!("{:30} {:>10} {:>10}", "TOTAL", "", total_overspent);
                println!();
                println!(
                    "⚠️  You have {} category/categories overspent by {} total.",
                    overspent.len(),
                    total_overspent
                );
                println!("Consider moving funds from other categories to cover the deficit.");
            }
        }
    }

    Ok(())
}
