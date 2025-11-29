# Expected Income Feature Implementation Guide

## Overview

This document provides a comprehensive guide for implementing the Expected Income feature in EnvelopeCLI. The feature allows users to set expected income per budget period and receive warnings when budgeted amounts exceed expected income.

## Architecture Decision

**Chosen Approach: Option A - Period-Based Income Expectations**

- Simple `IncomeExpectation` model tied to `BudgetPeriod`
- One expected income amount per period
- Warnings when total budgeted exceeds expected income
- Follows existing patterns in the codebase

## Implementation Tasks

### Phase 1: Data Model Layer

#### Task 1.1: Create IncomeId type (`src/models/ids.rs`)

Add a new strongly-typed ID for income expectations, following the existing pattern:

```rust
/// Unique identifier for an income expectation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IncomeId(uuid::Uuid);

impl IncomeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(uuid::Uuid::parse_str(s)?))
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl Default for IncomeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for IncomeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "inc-{}", &self.0.to_string()[..8])
    }
}
```

#### Task 1.2: Create Income Model (`src/models/income.rs`)

Create a new file with the `IncomeExpectation` struct:

```rust
//! Income expectation model
//!
//! Tracks expected income per budget period, allowing users to see
//! when they're budgeting more than they expect to earn.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids::IncomeId;
use super::money::Money;
use super::period::BudgetPeriod;

/// Validation errors for income expectations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncomeValidationError {
    NegativeAmount,
}

impl std::fmt::Display for IncomeValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NegativeAmount => write!(f, "Expected income cannot be negative"),
        }
    }
}

impl std::error::Error for IncomeValidationError {}

/// Expected income for a budget period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeExpectation {
    pub id: IncomeId,
    pub period: BudgetPeriod,
    pub expected_amount: Money,
    #[serde(default)]
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IncomeExpectation {
    /// Create a new income expectation
    pub fn new(period: BudgetPeriod, expected_amount: Money) -> Self {
        let now = Utc::now();
        Self {
            id: IncomeId::new(),
            period,
            expected_amount,
            notes: String::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the expected amount
    pub fn set_expected_amount(&mut self, amount: Money) {
        self.expected_amount = amount;
        self.updated_at = Utc::now();
    }

    /// Set notes
    pub fn set_notes(&mut self, notes: impl Into<String>) {
        self.notes = notes.into();
        self.updated_at = Utc::now();
    }

    /// Validate the income expectation
    pub fn validate(&self) -> Result<(), IncomeValidationError> {
        if self.expected_amount.is_negative() {
            return Err(IncomeValidationError::NegativeAmount);
        }
        Ok(())
    }

    /// Check if a budgeted amount exceeds expected income
    pub fn is_over_budget(&self, total_budgeted: Money) -> bool {
        total_budgeted > self.expected_amount
    }

    /// Get the difference between expected income and budgeted amount
    /// Positive = under budget (good), Negative = over budget (warning)
    pub fn budget_difference(&self, total_budgeted: Money) -> Money {
        self.expected_amount - total_budgeted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_income_expectation() {
        let period = BudgetPeriod::monthly(2025, 1);
        let income = IncomeExpectation::new(period.clone(), Money::from_cents(500000));

        assert_eq!(income.period, period);
        assert_eq!(income.expected_amount.cents(), 500000);
        assert!(income.notes.is_empty());
    }

    #[test]
    fn test_validation_negative_amount() {
        let period = BudgetPeriod::monthly(2025, 1);
        let income = IncomeExpectation::new(period, Money::from_cents(-100));

        assert!(matches!(
            income.validate(),
            Err(IncomeValidationError::NegativeAmount)
        ));
    }

    #[test]
    fn test_over_budget_detection() {
        let period = BudgetPeriod::monthly(2025, 1);
        let income = IncomeExpectation::new(period, Money::from_cents(500000)); // $5000

        // Under budget
        assert!(!income.is_over_budget(Money::from_cents(400000))); // $4000

        // Exactly at budget
        assert!(!income.is_over_budget(Money::from_cents(500000))); // $5000

        // Over budget
        assert!(income.is_over_budget(Money::from_cents(600000))); // $6000
    }

    #[test]
    fn test_budget_difference() {
        let period = BudgetPeriod::monthly(2025, 1);
        let income = IncomeExpectation::new(period, Money::from_cents(500000)); // $5000

        // Under budget by $1000
        let diff = income.budget_difference(Money::from_cents(400000));
        assert_eq!(diff.cents(), 100000);

        // Over budget by $1000
        let diff = income.budget_difference(Money::from_cents(600000));
        assert_eq!(diff.cents(), -100000);
    }
}
```

#### Task 1.3: Export from `src/models/mod.rs`

Add these lines:

```rust
pub mod income;
pub use income::IncomeExpectation;
pub use ids::IncomeId;
```

### Phase 2: Audit Layer

#### Task 2.1: Add Income to EntityType (`src/audit/mod.rs`)

Find the `EntityType` enum and add:

```rust
pub enum EntityType {
    Account,
    Transaction,
    Category,
    CategoryGroup,
    BudgetAllocation,
    BudgetTarget,
    Payee,
    IncomeExpectation,  // Add this
}
```

### Phase 3: Storage Layer

#### Task 3.1: Add income_file() to EnvelopePaths (`src/config/paths.rs`)

Find the `EnvelopePaths` impl and add:

```rust
/// Path to income expectations file
pub fn income_file(&self) -> PathBuf {
    self.data_dir().join("income.json")
}
```

#### Task 3.2: Create Income Repository (`src/storage/income.rs`)

```rust
//! Income expectations repository
//!
//! Handles persistence of income expectations to JSON files.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{BudgetPeriod, IncomeExpectation, IncomeId};
use crate::storage::file_io::{read_json, write_json_atomic};

/// Repository for income expectations
pub struct IncomeRepository {
    path: PathBuf,
    expectations: HashMap<BudgetPeriod, IncomeExpectation>,
}

impl IncomeRepository {
    /// Create a new repository
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            expectations: HashMap::new(),
        }
    }

    /// Load expectations from disk
    pub fn load(&mut self) -> EnvelopeResult<()> {
        if self.path.exists() {
            let list: Vec<IncomeExpectation> = read_json(&self.path)?;
            self.expectations = list.into_iter().map(|e| (e.period.clone(), e)).collect();
        }
        Ok(())
    }

    /// Save expectations to disk
    pub fn save(&self) -> EnvelopeResult<()> {
        let list: Vec<&IncomeExpectation> = self.expectations.values().collect();
        write_json_atomic(&self.path, &list)
    }

    /// Get income expectation for a period
    pub fn get_for_period(&self, period: &BudgetPeriod) -> Option<&IncomeExpectation> {
        self.expectations.get(period)
    }

    /// Get income expectation by ID
    pub fn get(&self, id: IncomeId) -> Option<&IncomeExpectation> {
        self.expectations.values().find(|e| e.id == id)
    }

    /// Upsert an income expectation (insert or update)
    pub fn upsert(&mut self, expectation: IncomeExpectation) -> EnvelopeResult<()> {
        self.expectations.insert(expectation.period.clone(), expectation);
        Ok(())
    }

    /// Delete income expectation for a period
    pub fn delete_for_period(&mut self, period: &BudgetPeriod) -> Option<IncomeExpectation> {
        self.expectations.remove(period)
    }

    /// Get all income expectations
    pub fn get_all(&self) -> Vec<&IncomeExpectation> {
        self.expectations.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Money;
    use tempfile::TempDir;

    #[test]
    fn test_upsert_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("income.json");
        let mut repo = IncomeRepository::new(path);

        let period = BudgetPeriod::monthly(2025, 1);
        let expectation = IncomeExpectation::new(period.clone(), Money::from_cents(500000));

        repo.upsert(expectation).unwrap();

        let retrieved = repo.get_for_period(&period).unwrap();
        assert_eq!(retrieved.expected_amount.cents(), 500000);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("income.json");

        // Save
        {
            let mut repo = IncomeRepository::new(path.clone());
            let period = BudgetPeriod::monthly(2025, 1);
            let expectation = IncomeExpectation::new(period, Money::from_cents(500000));
            repo.upsert(expectation).unwrap();
            repo.save().unwrap();
        }

        // Load
        {
            let mut repo = IncomeRepository::new(path);
            repo.load().unwrap();
            let period = BudgetPeriod::monthly(2025, 1);
            let retrieved = repo.get_for_period(&period).unwrap();
            assert_eq!(retrieved.expected_amount.cents(), 500000);
        }
    }
}
```

#### Task 3.3: Export from `src/storage/mod.rs`

Add:

```rust
pub mod income;
pub use income::IncomeRepository;
```

#### Task 3.4: Update Storage struct (`src/storage/mod.rs`)

Add to the `Storage` struct:

```rust
pub struct Storage {
    // ... existing fields
    pub income: IncomeRepository,
}
```

Update `Storage::new()`:

```rust
impl Storage {
    pub fn new(paths: EnvelopePaths) -> Result<Self, EnvelopeError> {
        // ... existing code
        Ok(Self {
            // ... existing fields
            income: IncomeRepository::new(paths.income_file()),
            // ...
        })
    }
}
```

#### Task 3.5: Update load_all() and save_all()

```rust
pub fn load_all(&mut self) -> Result<(), EnvelopeError> {
    // ... existing loads
    self.income.load()?;
    Ok(())
}

pub fn save_all(&self) -> Result<(), EnvelopeError> {
    // ... existing saves
    self.income.save()?;
    Ok(())
}
```

### Phase 4: Service Layer

#### Task 4.1: Create Income Service (`src/services/income.rs`)

```rust
//! Income service
//!
//! Provides business logic for managing expected income.

use crate::audit::EntityType;
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{BudgetPeriod, IncomeExpectation, Money};
use crate::storage::Storage;

/// Service for income expectation management
pub struct IncomeService<'a> {
    storage: &'a Storage,
}

impl<'a> IncomeService<'a> {
    /// Create a new income service
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Set expected income for a period
    pub fn set_expected_income(
        &self,
        period: &BudgetPeriod,
        amount: Money,
        notes: Option<String>,
    ) -> EnvelopeResult<IncomeExpectation> {
        let mut expectation = if let Some(existing) = self.storage.income.get_for_period(period) {
            let mut e = existing.clone();
            let before = e.clone();
            e.set_expected_amount(amount);
            if let Some(n) = notes {
                e.set_notes(n);
            }

            // Validate
            e.validate().map_err(|e| EnvelopeError::Income(e.to_string()))?;

            // Audit update
            self.storage.log_update(
                EntityType::IncomeExpectation,
                e.id.to_string(),
                Some(format!("Income for {}", period)),
                &before,
                &e,
                Some(format!("{} -> {}", before.expected_amount, e.expected_amount)),
            )?;

            e
        } else {
            let mut e = IncomeExpectation::new(period.clone(), amount);
            if let Some(n) = notes {
                e.set_notes(n);
            }

            // Validate
            e.validate().map_err(|e| EnvelopeError::Income(e.to_string()))?;

            // Audit create
            self.storage.log_create(
                EntityType::IncomeExpectation,
                e.id.to_string(),
                Some(format!("Income for {}", period)),
                &e,
            )?;

            e
        };

        // Save
        // Note: Need mutable access - see implementation notes below
        // self.storage.income.upsert(expectation.clone())?;
        // self.storage.income.save()?;

        Ok(expectation)
    }

    /// Get expected income for a period
    pub fn get_expected_income(&self, period: &BudgetPeriod) -> Option<Money> {
        self.storage
            .income
            .get_for_period(period)
            .map(|e| e.expected_amount)
    }

    /// Get the full income expectation for a period
    pub fn get_income_expectation(&self, period: &BudgetPeriod) -> Option<&IncomeExpectation> {
        self.storage.income.get_for_period(period)
    }

    /// Delete income expectation for a period
    pub fn delete_expected_income(&self, period: &BudgetPeriod) -> EnvelopeResult<bool> {
        // Implementation needs mutable storage access
        Ok(false)
    }
}
```

**Implementation Note:** The service pattern in this codebase uses `&Storage` (immutable reference). You'll need to either:
1. Change the service to take `&mut Storage` for mutation operations, OR
2. Have the CLI/TUI layer handle the mutation directly (like some existing code does)

Look at how `BudgetService::assign_to_category()` handles this - it calls `self.storage.budget.upsert()` which requires the repository methods to use interior mutability or the caller to handle it.

#### Task 4.2: Export from `src/services/mod.rs`

```rust
pub mod income;
pub use income::IncomeService;
```

#### Task 4.3: Update BudgetService (`src/services/budget.rs`)

Add method to get expected income and check over-budget:

```rust
impl<'a> BudgetService<'a> {
    // ... existing methods

    /// Get expected income for a period (if set)
    pub fn get_expected_income(&self, period: &BudgetPeriod) -> Option<Money> {
        self.storage
            .income
            .get_for_period(period)
            .map(|e| e.expected_amount)
    }

    /// Check if total budgeted exceeds expected income
    pub fn is_over_expected_income(&self, period: &BudgetPeriod) -> EnvelopeResult<Option<Money>> {
        let expected = match self.get_expected_income(period) {
            Some(e) => e,
            None => return Ok(None), // No expectation set
        };

        let allocations = self.storage.budget.get_for_period(period)?;
        let total_budgeted: Money = allocations.iter().map(|a| a.budgeted).sum();

        if total_budgeted > expected {
            Ok(Some(total_budgeted - expected)) // Return overage amount
        } else {
            Ok(None)
        }
    }
}
```

#### Task 4.4: Update BudgetOverview struct

Add income-related fields:

```rust
/// Budget overview for a period
#[derive(Debug, Clone)]
pub struct BudgetOverview {
    pub period: BudgetPeriod,
    pub total_budgeted: Money,
    pub total_activity: Money,
    pub total_available: Money,
    pub available_to_budget: Money,
    pub categories: Vec<CategoryBudgetSummary>,
    // New fields
    pub expected_income: Option<Money>,
    pub over_budget_amount: Option<Money>, // Some if budgeted > expected
}
```

Update `get_budget_overview()` to populate these fields.

### Phase 5: CLI Layer

#### Task 5.1: Create Income CLI (`src/cli/income.rs`)

```rust
//! Income CLI commands

use clap::{Parser, Subcommand};

use crate::error::EnvelopeResult;
use crate::models::{BudgetPeriod, Money};
use crate::services::{BudgetService, IncomeService};
use crate::storage::Storage;

#[derive(Subcommand)]
pub enum IncomeCommands {
    /// Set expected income for a period
    Set {
        /// Expected income amount (e.g., 5000.00)
        amount: String,

        /// Budget period (e.g., 2025-01 for January 2025)
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

pub fn handle_income_command(cmd: IncomeCommands, storage: &mut Storage) -> EnvelopeResult<()> {
    match cmd {
        IncomeCommands::Set { amount, period, notes } => {
            let period = parse_period_or_current(period.as_deref())?;
            let amount = Money::parse(&amount)?;

            // Create/update income expectation
            let mut expectation = crate::models::IncomeExpectation::new(period.clone(), amount);
            if let Some(n) = notes {
                expectation.set_notes(n);
            }
            expectation.validate().map_err(|e| crate::error::EnvelopeError::Income(e.to_string()))?;

            storage.income.upsert(expectation)?;
            storage.income.save()?;

            println!("✓ Set expected income for {} to {}", period, amount);

            // Show comparison if budget exists
            let budget_service = BudgetService::new(storage);
            if let Some(overage) = budget_service.is_over_expected_income(&period)? {
                println!("⚠️  Warning: You're budgeting {} more than expected income!", overage);
            }

            Ok(())
        }

        IncomeCommands::Show { period } => {
            let period = parse_period_or_current(period.as_deref())?;

            if let Some(expectation) = storage.income.get_for_period(&period) {
                println!("Expected Income for {}", period);
                println!("─────────────────────────────");
                println!("Amount: {}", expectation.expected_amount);
                if !expectation.notes.is_empty() {
                    println!("Notes:  {}", expectation.notes);
                }

                // Show budget comparison
                let budget_service = BudgetService::new(storage);
                let overview = budget_service.get_budget_overview(&period)?;
                println!();
                println!("Budget Comparison:");
                println!("  Expected Income:  {}", expectation.expected_amount);
                println!("  Total Budgeted:   {}", overview.total_budgeted);
                let diff = expectation.expected_amount - overview.total_budgeted;
                if diff.is_negative() {
                    println!("  Over Budget:      {} ⚠️", -diff);
                } else {
                    println!("  Remaining:        {} ✓", diff);
                }
            } else {
                println!("No expected income set for {}", period);
                println!("Use 'envelope income set <amount>' to set expected income.");
            }

            Ok(())
        }

        IncomeCommands::Remove { period } => {
            let period = parse_period_or_current(period.as_deref())?;

            if storage.income.delete_for_period(&period).is_some() {
                storage.income.save()?;
                println!("✓ Removed expected income for {}", period);
            } else {
                println!("No expected income was set for {}", period);
            }

            Ok(())
        }

        IncomeCommands::Compare { period } => {
            let period = parse_period_or_current(period.as_deref())?;
            let budget_service = BudgetService::new(storage);
            let overview = budget_service.get_budget_overview(&period)?;

            println!("Income vs Budget Comparison for {}", period);
            println!("═══════════════════════════════════════════");

            if let Some(expectation) = storage.income.get_for_period(&period) {
                println!("Expected Income:     {:>12}", expectation.expected_amount);
                println!("Total Budgeted:      {:>12}", overview.total_budgeted);
                println!("───────────────────────────────────────────");

                let diff = expectation.expected_amount - overview.total_budgeted;
                if diff.is_negative() {
                    println!("OVER BUDGET:         {:>12} ⚠️", -diff);
                    println!();
                    println!("⚠️  You're budgeting more than you expect to earn!");
                    println!("   Consider reducing budget allocations or increasing expected income.");
                } else if diff.is_zero() {
                    println!("Remaining to Budget: {:>12} ✓", diff);
                    println!();
                    println!("✓ Your budget exactly matches expected income.");
                } else {
                    println!("Remaining to Budget: {:>12} ✓", diff);
                    println!();
                    println!("✓ You have {} available to budget.", diff);
                }
            } else {
                println!("Expected Income:     Not set");
                println!("Total Budgeted:      {:>12}", overview.total_budgeted);
                println!();
                println!("Tip: Set expected income with 'envelope income set <amount>'");
            }

            Ok(())
        }
    }
}

fn parse_period_or_current(period_str: Option<&str>) -> EnvelopeResult<BudgetPeriod> {
    match period_str {
        Some(s) => BudgetPeriod::parse(s),
        None => Ok(BudgetPeriod::current_month()),
    }
}
```

#### Task 5.2: Export from `src/cli/mod.rs`

```rust
pub mod income;
pub use income::{handle_income_command, IncomeCommands};
```

#### Task 5.3: Add to main Commands enum (`src/main.rs`)

Find the `Commands` enum and add:

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands

    /// Manage expected income
    Income {
        #[command(subcommand)]
        command: IncomeCommands,
    },
}
```

In the match statement:

```rust
Commands::Income { command } => handle_income_command(command, &mut storage)?,
```

#### Task 5.4: Update Budget Overview CLI (`src/cli/budget.rs`)

Find the overview command and enhance the output to show income comparison when available.

### Phase 6: TUI Layer

#### Task 6.1: Add Income to Budget View Header (`src/tui/views/budget.rs`)

Update the header rendering to show expected income when set:

```
┌─────────────────────────────────────────────────────────────┐
│ January 2025                                                │
│ Expected Income: $5,000.00  |  Budgeted: $4,800.00  ✓       │
│ Available to Budget: $200.00                                │
└─────────────────────────────────────────────────────────────┘
```

Or when over-budgeted:

```
┌─────────────────────────────────────────────────────────────┐
│ January 2025                                                │
│ Expected Income: $5,000.00  |  Budgeted: $5,500.00  ⚠️      │
│ OVER BUDGET BY: $500.00                                     │
└─────────────────────────────────────────────────────────────┘
```

#### Task 6.2: Add Warning Indicator

When `total_budgeted > expected_income`, show:
- Red/yellow color on the header
- Warning icon (⚠️)
- "OVER BUDGET BY: $X" message

### Phase 7: Error Handling

#### Task 7.1: Add Income Error Variant (`src/error.rs`)

```rust
pub enum EnvelopeError {
    // ... existing variants
    #[error("Income error: {0}")]
    Income(String),
}
```

### Phase 8: Testing

#### Task 8.1: Model Tests (in `src/models/income.rs`)

- Test creation
- Test validation (negative amounts)
- Test over-budget detection
- Test budget difference calculation

#### Task 8.2: Repository Tests (in `src/storage/income.rs`)

- Test upsert and get
- Test save and load persistence
- Test delete

#### Task 8.3: Service Tests (in `src/services/income.rs`)

- Test setting income
- Test getting income
- Test budget comparison

#### Task 8.4: Integration Tests

Create `tests/income_integration.rs`:
- Test full workflow: set income → budget → check comparison
- Test CLI commands work correctly

### Phase 9: Final Steps

1. Run `cargo build` and fix any compilation errors
2. Run `cargo test` and ensure all tests pass
3. Run `cargo clippy` and address any warnings
4. Test manually:
   ```bash
   envelope income set 5000.00
   envelope income show
   envelope budget assign "Groceries" 2000.00
   envelope income compare
   ```

## File Summary

| File | Action | Description |
|------|--------|-------------|
| `src/models/ids.rs` | Modify | Add `IncomeId` type |
| `src/models/income.rs` | Create | `IncomeExpectation` model |
| `src/models/mod.rs` | Modify | Export income module |
| `src/audit/mod.rs` | Modify | Add `IncomeExpectation` to `EntityType` |
| `src/config/paths.rs` | Modify | Add `income_file()` method |
| `src/storage/income.rs` | Create | `IncomeRepository` |
| `src/storage/mod.rs` | Modify | Export income repo, add to `Storage` |
| `src/services/income.rs` | Create | `IncomeService` |
| `src/services/mod.rs` | Modify | Export income service |
| `src/services/budget.rs` | Modify | Add income-related methods |
| `src/cli/income.rs` | Create | Income CLI commands |
| `src/cli/mod.rs` | Modify | Export income CLI |
| `src/main.rs` | Modify | Add `Income` command |
| `src/cli/budget.rs` | Modify | Show income in overview |
| `src/tui/views/budget.rs` | Modify | Show income in TUI header |
| `src/error.rs` | Modify | Add `Income` error variant |

## Dependencies

No new crate dependencies required - all functionality uses existing dependencies (serde, chrono, uuid, etc.).
