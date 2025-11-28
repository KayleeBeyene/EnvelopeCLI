//! Budget period setup step
//!
//! Allows users to select their preferred budget period type.

use std::io::{self, Write};

use crate::config::settings::BudgetPeriodType;
use crate::error::EnvelopeResult;

/// Period setup step result
pub struct PeriodSetupResult {
    /// The user's preferred period type
    pub period_type: BudgetPeriodType,
}

/// Period setup step
pub struct PeriodSetupStep;

impl PeriodSetupStep {
    /// Run the period setup step
    pub fn run() -> EnvelopeResult<PeriodSetupResult> {
        println!();
        println!("Step 3: Budget Period");
        println!("=====================");
        println!();
        println!("How often would you like to budget?");
        println!();
        println!("  1. Monthly (recommended) - Budget once per month");
        println!("  2. Weekly - Budget every week");
        println!("  3. Bi-weekly - Budget every two weeks");
        println!();

        let choice_str = prompt_string("Select budget period [1]: ")?;
        let period_type = match choice_str.trim() {
            "" | "1" => BudgetPeriodType::Monthly,
            "2" => BudgetPeriodType::Weekly,
            "3" => BudgetPeriodType::BiWeekly,
            _ => BudgetPeriodType::Monthly,
        };

        println!();
        match period_type {
            BudgetPeriodType::Monthly => {
                println!("You've selected monthly budgeting.");
                println!("Your budget will reset at the beginning of each month.");
            }
            BudgetPeriodType::Weekly => {
                println!("You've selected weekly budgeting.");
                println!("Your budget will reset every week.");
            }
            BudgetPeriodType::BiWeekly => {
                println!("You've selected bi-weekly budgeting.");
                println!("Your budget will reset every two weeks.");
            }
        }

        Ok(PeriodSetupResult { period_type })
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
