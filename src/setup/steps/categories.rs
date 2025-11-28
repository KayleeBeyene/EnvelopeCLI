//! Categories setup step
//!
//! Allows users to select or customize their category groups.

use std::io::{self, Write};

use crate::error::EnvelopeResult;

/// Category setup choice
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryChoice {
    /// Use default categories
    UseDefaults,
    /// Customize categories (future enhancement)
    Customize,
    /// Start with empty categories
    Empty,
}

/// Categories setup step result
pub struct CategoriesSetupResult {
    /// The user's category choice
    pub choice: CategoryChoice,
}

/// Categories setup step
pub struct CategoriesSetupStep;

impl CategoriesSetupStep {
    /// Run the categories setup step
    pub fn run() -> EnvelopeResult<CategoriesSetupResult> {
        println!();
        println!("Step 2: Category Groups");
        println!("=======================");
        println!();
        println!("EnvelopeCLI organizes your budget into category groups.");
        println!();
        println!("Default groups include:");
        println!("  - Bills: Rent/Mortgage, Electric, Water, Internet, Phone, Insurance");
        println!("  - Needs: Groceries, Transportation, Medical, Household");
        println!("  - Wants: Dining Out, Entertainment, Shopping, Subscriptions");
        println!("  - Savings: Emergency Fund, Vacation, Large Purchases");
        println!();
        println!("What would you like to do?");
        println!("  1. Use default categories (recommended)");
        println!("  2. Start with empty categories (add your own later)");
        println!();

        let choice_str = prompt_string("Select option [1]: ")?;
        let choice = match choice_str.trim() {
            "" | "1" => CategoryChoice::UseDefaults,
            "2" => CategoryChoice::Empty,
            _ => CategoryChoice::UseDefaults,
        };

        match choice {
            CategoryChoice::UseDefaults => {
                println!();
                println!("Default categories will be created.");
            }
            CategoryChoice::Empty => {
                println!();
                println!("Starting with empty categories.");
                println!("Use 'envelope category create' to add categories later.");
            }
            CategoryChoice::Customize => {
                // Future enhancement
                println!();
                println!("Category customization will be available in a future update.");
                println!("Using default categories for now.");
            }
        }

        Ok(CategoriesSetupResult { choice })
    }
}

/// Prompt for a string input
fn prompt_string(prompt: &str) -> EnvelopeResult<String> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}
