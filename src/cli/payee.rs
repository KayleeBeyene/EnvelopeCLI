//! Payee CLI commands
//!
//! Implements CLI commands for payee management.

use clap::Subcommand;

use crate::error::{EnvelopeError, EnvelopeResult};
use crate::services::{CategoryService, PayeeService};
use crate::storage::Storage;

/// Payee subcommands
#[derive(Subcommand)]
pub enum PayeeCommands {
    /// List all payees
    List {
        /// Search query to filter payees
        #[arg(short, long)]
        search: Option<String>,
    },
    /// Show payee details
    Show {
        /// Payee name or ID
        payee: String,
    },
    /// Set default category for a payee
    SetCategory {
        /// Payee name or ID
        payee: String,
        /// Category name
        category: String,
    },
    /// Clear default category for a payee
    ClearCategory {
        /// Payee name or ID
        payee: String,
    },
    /// Delete a payee
    Delete {
        /// Payee name or ID
        payee: String,
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Rename a payee
    Rename {
        /// Payee name or ID
        payee: String,
        /// New name
        name: String,
    },
}

/// Handle a payee command
pub fn handle_payee_command(storage: &Storage, cmd: PayeeCommands) -> EnvelopeResult<()> {
    let service = PayeeService::new(storage);
    let category_service = CategoryService::new(storage);

    match cmd {
        PayeeCommands::List { search } => {
            let payees = if let Some(query) = search {
                service.search(&query, 50)?
            } else {
                service.list()?
            };

            if payees.is_empty() {
                println!("No payees found.");
                return Ok(());
            }

            println!("{:30} {:20} {:10}", "Name", "Default Category", "Usage");
            println!("{}", "-".repeat(62));

            for payee in &payees {
                let cat_name = if let Some(cat_id) = payee.default_category_id {
                    category_service
                        .get_category(cat_id)?
                        .map(|c| c.name)
                        .unwrap_or_else(|| "(deleted)".to_string())
                } else {
                    "(auto)".to_string()
                };

                let usage: u32 = payee.category_frequency.values().sum();
                let manual_indicator = if payee.manual { "*" } else { "" };

                println!(
                    "{:30} {:20} {:>10}{}",
                    truncate(&payee.name, 30),
                    truncate(&cat_name, 20),
                    usage,
                    manual_indicator
                );
            }

            println!("\nTotal: {} payees", payees.len());
            println!("* = manually configured default category");
        }

        PayeeCommands::Show { payee } => {
            let p = service
                .find(&payee)?
                .ok_or_else(|| EnvelopeError::payee_not_found(&payee))?;

            println!("Payee: {}", p.name);
            println!("ID:    {}", p.id);

            if let Some(cat_id) = p.default_category_id {
                if let Some(cat) = category_service.get_category(cat_id)? {
                    let source = if p.manual { "manual" } else { "learned" };
                    println!("Default Category: {} ({})", cat.name, source);
                }
            } else if let Some(suggested) = p.suggested_category() {
                if let Some(cat) = category_service.get_category(suggested)? {
                    println!("Suggested Category: {} (learned)", cat.name);
                }
            } else {
                println!("Default Category: (none)");
            }

            if !p.category_frequency.is_empty() {
                println!("\nCategory Usage:");
                let mut freq: Vec<_> = p.category_frequency.iter().collect();
                freq.sort_by(|a, b| b.1.cmp(a.1));

                for (cat_id, count) in freq.iter().take(5) {
                    if let Some(cat) = category_service.get_category(**cat_id)? {
                        println!("  {:20} {:>5} times", cat.name, count);
                    }
                }
            }

            println!("\nCreated:  {}", p.created_at.format("%Y-%m-%d %H:%M"));
            println!("Updated:  {}", p.updated_at.format("%Y-%m-%d %H:%M"));
        }

        PayeeCommands::SetCategory { payee, category } => {
            let p = service
                .find(&payee)?
                .ok_or_else(|| EnvelopeError::payee_not_found(&payee))?;

            let cat = category_service
                .find_category(&category)?
                .ok_or_else(|| EnvelopeError::category_not_found(&category))?;

            let updated = service.set_default_category(p.id, cat.id)?;
            println!(
                "Set default category for '{}' to '{}'",
                updated.name, cat.name
            );
        }

        PayeeCommands::ClearCategory { payee } => {
            let p = service
                .find(&payee)?
                .ok_or_else(|| EnvelopeError::payee_not_found(&payee))?;

            let updated = service.clear_default_category(p.id)?;
            println!(
                "Cleared default category for '{}' (will use learned suggestions)",
                updated.name
            );
        }

        PayeeCommands::Delete { payee, force } => {
            let p = service
                .find(&payee)?
                .ok_or_else(|| EnvelopeError::payee_not_found(&payee))?;

            if !force {
                println!("About to delete payee: {}", p.name);
                println!("Use --force to confirm deletion");
                return Ok(());
            }

            let deleted = service.delete(p.id)?;
            println!("Deleted payee: {}", deleted.name);
        }

        PayeeCommands::Rename { payee, name } => {
            let p = service
                .find(&payee)?
                .ok_or_else(|| EnvelopeError::payee_not_found(&payee))?;

            let old_name = p.name.clone();
            let renamed = service.rename(p.id, &name)?;
            println!("Renamed payee: '{}' -> '{}'", old_name, renamed.name);
        }
    }

    Ok(())
}

/// Truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
