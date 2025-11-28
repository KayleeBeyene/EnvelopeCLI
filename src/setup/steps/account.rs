//! Account setup step
//!
//! Creates the user's first account with an optional starting balance.

use std::io::{self, Write};

use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::account::{Account, AccountType};
use crate::models::Money;

/// Account setup step result
pub struct AccountSetupResult {
    /// The created account
    pub account: Account,
    /// Starting balance as income (to be added to Available to Budget)
    pub starting_balance: Money,
}

/// Account setup step
pub struct AccountSetupStep;

impl AccountSetupStep {
    /// Run the account setup step
    pub fn run() -> EnvelopeResult<AccountSetupResult> {
        println!();
        println!("Step 1: Create Your First Account");
        println!("==================================");
        println!();
        println!("Let's create your first account. This is typically your main checking");
        println!("account that you use for everyday spending.");
        println!();

        // Get account name
        let name = prompt_string("Account name (e.g., 'Checking', 'Main Account'): ")?;
        if name.is_empty() {
            return Err(EnvelopeError::Validation("Account name cannot be empty".into()));
        }

        // Get account type
        println!();
        println!("Account type:");
        println!("  1. Checking (default)");
        println!("  2. Savings");
        println!("  3. Credit Card");
        println!("  4. Cash");
        println!("  5. Other");
        let type_choice = prompt_string("Select account type [1]: ")?;
        let account_type = match type_choice.trim() {
            "" | "1" => AccountType::Checking,
            "2" => AccountType::Savings,
            "3" => AccountType::Credit,
            "4" => AccountType::Cash,
            "5" => AccountType::Other,
            _ => AccountType::Checking,
        };

        // Get starting balance
        println!();
        println!("What is the current balance of this account?");
        println!("(This will be added to your 'Available to Budget')");
        let balance_str = prompt_string("Starting balance (e.g., 1000.00): ")?;
        let starting_balance = if balance_str.is_empty() {
            Money::zero()
        } else {
            Money::parse(&balance_str).map_err(|e| {
                EnvelopeError::Validation(format!("Invalid amount: {}", e))
            })?
        };

        // Create the account
        let account = Account::new(name, account_type);

        println!();
        println!("Account '{}' will be created with balance {}",
            account.name, starting_balance);

        Ok(AccountSetupResult {
            account,
            starting_balance,
        })
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
