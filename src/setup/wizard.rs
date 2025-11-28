//! Setup wizard orchestration
//!
//! Coordinates the multi-step setup process for first-time users.

use std::io::{self, Write};

use crate::config::{paths::EnvelopePaths, settings::Settings};
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{Account, Money, TransactionStatus};
use crate::services::account::AccountService;
use crate::services::transaction::{CreateTransactionInput, TransactionService};
use crate::storage::Storage;

use super::steps::{
    account::AccountSetupStep,
    categories::{CategoriesSetupStep, CategoryChoice},
    period::PeriodSetupStep,
};

/// Result of running the setup wizard
pub struct SetupResult {
    /// Whether setup was completed successfully
    pub completed: bool,
    /// The created account (if any)
    pub account: Option<Account>,
    /// Starting balance added to Available to Budget
    pub starting_balance: Money,
}

/// The setup wizard state machine
pub struct SetupWizard {
    paths: EnvelopePaths,
}

impl SetupWizard {
    /// Create a new setup wizard
    pub fn new(paths: EnvelopePaths) -> Self {
        Self { paths }
    }

    /// Check if setup is needed (first run)
    pub fn needs_setup(&self, settings: &Settings) -> bool {
        !settings.setup_completed && !self.paths.settings_file().exists()
    }

    /// Run the interactive setup wizard
    pub fn run(
        &self,
        storage: &Storage,
        settings: &mut Settings,
    ) -> EnvelopeResult<SetupResult> {
        println!();
        println!("===========================================");
        println!("  Welcome to EnvelopeCLI Setup Wizard!");
        println!("===========================================");
        println!();
        println!("This wizard will help you set up your budget.");
        println!("Press Ctrl+C at any time to cancel.");
        println!();

        // Confirm start
        let confirm = prompt_string("Ready to begin? (yes/no) [yes]: ")?;
        if !confirm.is_empty() && confirm.to_lowercase() != "yes" && confirm.to_lowercase() != "y" {
            println!("Setup cancelled.");
            return Ok(SetupResult {
                completed: false,
                account: None,
                starting_balance: Money::zero(),
            });
        }

        // Step 1: Create first account
        let account_result = AccountSetupStep::run()?;

        // Step 2: Category groups
        let categories_result = CategoriesSetupStep::run()?;

        // Step 3: Budget period
        let period_result = PeriodSetupStep::run()?;

        // Summary
        println!();
        println!("===========================================");
        println!("  Setup Summary");
        println!("===========================================");
        println!();
        println!("Account: {} ({})", account_result.account.name, account_result.account.account_type);
        println!("Starting Balance: {}", account_result.starting_balance);
        println!("Categories: {}", match categories_result.choice {
            CategoryChoice::UseDefaults => "Default categories",
            CategoryChoice::Empty => "Empty (add your own)",
            CategoryChoice::Customize => "Custom",
        });
        println!("Budget Period: {:?}", period_result.period_type);
        println!();

        let confirm = prompt_string("Apply these settings? (yes/no) [yes]: ")?;
        if !confirm.is_empty() && confirm.to_lowercase() != "yes" && confirm.to_lowercase() != "y" {
            println!("Setup cancelled.");
            return Ok(SetupResult {
                completed: false,
                account: None,
                starting_balance: Money::zero(),
            });
        }

        // Apply settings
        println!();
        println!("Applying settings...");

        // Initialize storage with defaults if using default categories
        if categories_result.choice == CategoryChoice::UseDefaults {
            crate::storage::init::initialize_storage(&self.paths)?;
        }

        // Save the account
        let account_service = AccountService::new(storage);
        let saved_account = account_service.create(
            &account_result.account.name,
            account_result.account.account_type,
            account_result.starting_balance,
            account_result.account.on_budget,
        )?;

        // Create starting balance transaction if non-zero
        if !account_result.starting_balance.is_zero() {
            let txn_service = TransactionService::new(storage);
            let input = CreateTransactionInput {
                account_id: saved_account.id.clone(),
                date: chrono::Local::now().naive_local().date(),
                amount: account_result.starting_balance,
                payee_name: Some("Starting Balance".to_string()),
                category_id: None,
                memo: Some("Initial account balance".to_string()),
                status: Some(TransactionStatus::Cleared),
            };
            txn_service.create(input)?;
        }

        // Update settings
        settings.budget_period_type = period_result.period_type;
        settings.setup_completed = true;
        settings.save(&self.paths)?;

        println!();
        println!("Setup complete!");
        println!();
        println!("Your budget is ready. Here are some next steps:");
        println!("  - Run 'envelope tui' to open the interactive interface");
        println!("  - Run 'envelope budget assign' to allocate funds to categories");
        println!("  - Run 'envelope transaction add' to record transactions");
        println!();

        Ok(SetupResult {
            completed: true,
            account: Some(saved_account),
            starting_balance: account_result.starting_balance,
        })
    }

    /// Run a minimal CLI setup (non-interactive)
    pub fn run_minimal(
        &self,
        _storage: &Storage,
        settings: &mut Settings,
    ) -> EnvelopeResult<SetupResult> {
        println!("Initializing EnvelopeCLI...");

        // Initialize default categories
        crate::storage::init::initialize_storage(&self.paths)?;

        // Mark setup as complete
        settings.setup_completed = true;
        settings.save(&self.paths)?;

        println!("Initialization complete!");

        Ok(SetupResult {
            completed: true,
            account: None,
            starting_balance: Money::zero(),
        })
    }
}

/// Prompt for a string input
fn prompt_string(prompt: &str) -> EnvelopeResult<String> {
    print!("{}", prompt);
    io::stdout().flush().map_err(|e| EnvelopeError::Io(e.to_string()))?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| EnvelopeError::Io(e.to_string()))?;

    Ok(input.trim().to_string())
}
