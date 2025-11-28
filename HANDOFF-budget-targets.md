# Budget Targets Feature - Implementation Handoff

**Date:** 2025-11-28
**Branch:** `feature/recurring-budget-targets`
**Status:** Partially complete - TUI done, CLI commands remaining

## Overview

This feature adds YNAB-style recurring budget targets to EnvelopeCLI. Users can set budget targets with various cadences (Weekly, Monthly, Yearly, Custom intervals, or By-Date goals) that automatically calculate the suggested budget amount for any given period.

## Completed Work

### 1. Core Models (`src/models/target.rs`)
- `BudgetTargetId` - Unique identifier (UUID-based)
- `TargetCadence` enum with variants:
  - `Weekly` - Budget target resets every week
  - `Monthly` - Budget target resets every month
  - `Yearly` - Budget target resets every year
  - `Custom { days: u32 }` - Budget target resets every N days
  - `ByDate { target_date: NaiveDate }` - Accumulate towards a specific date
- `BudgetTarget` struct with:
  - `id`, `category_id`, `amount`, `cadence`, `notes`, `active`
  - `created_at`, `updated_at` timestamps
  - `calculate_for_period(&BudgetPeriod)` - Converts target to period-specific amount
- `TargetValidationError` enum for validation errors

### 2. Storage (`src/storage/targets.rs`)
- `TargetRepository` - JSON-based persistence with RwLock<HashMap>
- Methods: `load()`, `save()`, `get()`, `get_for_category()`, `get_all_active()`, `upsert()`, `delete()`
- Data stored in `targets.json` in the data directory

### 3. Storage Integration
- **`src/storage/mod.rs`**: Added `targets: TargetRepository` to `Storage` struct
- **`src/config/paths.rs`**: Added `targets_file()` method
- **`src/audit/entry.rs`**: Added `BudgetTarget` to `EntityType` enum

### 4. Budget Service Methods (`src/services/budget.rs`)
Added the following methods:
- `set_target()` - Create or update a budget target for a category
- `update_target()` - Update an existing target
- `get_target()` - Get target for a category
- `get_suggested_budget()` - Calculate suggested amount for a period based on target
- `delete_target()` / `remove_target()` - Delete a target
- `get_all_targets()` - List all active targets
- `auto_fill_from_target()` - Auto-fill budget from target for one category
- `auto_fill_all_targets()` - Auto-fill all budgets from targets

### 5. TUI Dialog (`src/tui/dialogs/target.rs`)
- `TargetFormState` - Form state management
- `CadenceOption` enum for UI selection
- `TargetField` enum for field navigation
- Features:
  - Amount input with cursor editing
  - Cadence dropdown (j/k to cycle, Enter to confirm)
  - Dynamic fields for Custom (days input) and ByDate (date picker)
  - Error display
  - Edit existing targets or create new ones

### 6. TUI Integration
- **`src/tui/app.rs`**:
  - Added `ActiveDialog::SetTarget` variant
  - Added `target_form: TargetFormState` to App
  - Added initialization in `open_dialog()` for SetTarget
- **`src/tui/handler.rs`**:
  - Added 't' key binding in budget view to open SetTarget dialog
  - Added SetTarget dialog key handling
- **`src/tui/views/mod.rs`**: Added SetTarget dialog rendering case
- **`src/tui/dialogs/mod.rs`**: Added target module export

### 7. Edit Budget Dialog Enhancement (`src/tui/dialogs/edit_budget.rs`)
- Added `suggested_amount: Option<Money>` field to `EditBudgetState`
- Updated `init()` to accept suggested amount
- Added `use_suggested()` method
- Updated render to show suggested amount in green when available
- Added Tab key to fill in suggested amount
- Dynamic dialog height based on whether suggested amount exists

### 8. Tests
- Unit tests in `src/models/target.rs`:
  - `test_new_target`
  - `test_monthly_target_for_monthly_period`
  - `test_yearly_target_for_monthly_period`
  - `test_validation`
  - `test_serialization`

## Remaining Work

### 1. CLI Commands for Target Management (Priority: High)
Need to add CLI commands in `src/cli/` for:
- `envelope target set <category> <amount> [--cadence weekly|monthly|yearly|custom|by-date] [--days N] [--date YYYY-MM-DD]`
- `envelope target list` - List all targets
- `envelope target show <category>` - Show target for a category
- `envelope target delete <category>` - Remove target from a category
- `envelope target auto-fill [--period YYYY-MM]` - Auto-fill budgets from targets

Reference existing CLI structure in `src/cli/commands/` for patterns.

### 2. Budget View Enhancement (Priority: Medium)
Consider showing target indicators in the budget view:
- Show a small icon or indicator next to categories that have targets
- Maybe show progress towards By-Date goals

### 3. Additional Tests (Priority: Low)
- Integration tests for CLI commands once implemented
- More edge case tests for period calculations (leap years, month boundaries)
- Tests for `ByDate` cadence calculations

## Key Files Reference

| File | Purpose |
|------|---------|
| `src/models/target.rs` | Core BudgetTarget model and TargetCadence enum |
| `src/storage/targets.rs` | TargetRepository for JSON persistence |
| `src/services/budget.rs` | BudgetService with target calculation methods |
| `src/tui/dialogs/target.rs` | TUI dialog for setting targets |
| `src/tui/dialogs/edit_budget.rs` | Edit budget dialog with suggested amount |
| `src/tui/handler.rs:467-476` | 't' key binding for target dialog |
| `src/tui/handler.rs:459-466,684-690` | EditBudget init with suggested amount |

## How Targets Work

1. **Setting a Target**: User presses 't' on a category in budget view, enters amount and cadence
2. **Calculation**: When viewing a period, `calculate_for_period()` converts the target:
   - Weekly target in monthly view: amount × (days in month / 7)
   - Monthly target in weekly view: amount / 4.33
   - Yearly target in monthly view: amount / 12
   - ByDate: Distributes remaining amount over months until target date
3. **Suggestion**: When editing budget, suggested amount is shown if target exists
4. **Auto-fill**: Can automatically set budgets to match targets

## Build & Test

```bash
# Check compilation
cargo check

# Run all tests (288 should pass)
cargo test

# Run specific target tests
cargo test target
```

## Notes for Next Instance

- All code compiles and tests pass as of handoff
- The Money type doesn't implement Mul/Div traits - use `Money::from_cents()` for calculations
- The linter may delete newly created files - be careful and verify files exist after running
- Current branch has uncommitted changes - consider committing before making more changes
