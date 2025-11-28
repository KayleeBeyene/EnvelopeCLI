# Budget Targets Feature - Implementation Handoff

**Date:** 2025-11-28
**Branch:** `feature/recurring-budget-targets`
**Status:** ✅ COMPLETE - All features implemented and tested

## Overview

This feature adds YNAB-style recurring budget targets to EnvelopeCLI. Users can set budget targets with various cadences (Weekly, Monthly, Yearly, Custom intervals, or By-Date goals) that automatically calculate the suggested budget amount for any given period.

## Completed Work (All Verified ✅)

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
- Comprehensive test suite (40+ tests covering all cadences, edge cases, serialization)

### 2. Storage (`src/storage/targets.rs`)
- `TargetRepository` - JSON-based persistence with RwLock<HashMap>
- Methods: `load()`, `save()`, `get()`, `get_for_category()`, `get_all_active()`, `upsert()`, `delete()`
- Data stored in `targets.json` in the data directory

### 3. Storage Integration
- **`src/storage/mod.rs`**: Added `targets: TargetRepository` to `Storage` struct
- **`src/config/paths.rs`**: Added `targets_file()` method
- **`src/audit/entry.rs`**: Added `BudgetTarget` to `EntityType` enum

### 4. Budget Service Methods (`src/services/budget.rs`)
All methods implemented and functional (lines 450-610):
- `set_target()` - Create or update a budget target for a category
- `update_target()` - Update an existing target
- `get_target()` - Get target for a category
- `get_suggested_budget()` - Calculate suggested amount for a period based on target
- `delete_target()` / `remove_target()` - Delete a target
- `get_all_targets()` - List all active targets
- `auto_fill_from_target()` - Auto-fill budget from target for one category
- `auto_fill_all_targets()` - Auto-fill all budgets from targets

### 5. Unified Budget Dialog (`src/tui/dialogs/budget.rs`)
Fully implemented tabbed dialog combining period budget and target settings:
- `BudgetDialogState` - Full state management for both tabs
- `BudgetTab` enum (Period, Target) with Tab key switching
- `TargetField` enum for field navigation (Amount, Cadence, CustomDays, TargetDate)
- `CadenceOption` enum for UI selection (Weekly, Monthly, Yearly, Custom, ByDate)
- Features:
  - Period Tab: Amount input, suggested amount display (green), [s] to use suggested
  - Target Tab: Amount input, cadence selector (j/k to cycle), dynamic fields for Custom/ByDate
  - Error display, cursor editing, [Del] to remove target
  - Dynamic dialog height based on content

### 6. TUI Integration
- **`src/tui/app.rs`**: `budget_dialog_state: BudgetDialogState` in App struct
- **`src/tui/handler.rs`**: `Enter`, `b`, and `t` keys open unified budget dialog in budget view
- **`src/tui/views/budget.rs`**: Target indicators (◉) and progress display for categories with targets

### 7. CLI Commands (`src/cli/target.rs`)
Full CLI implementation:
- `envelope target set <category> <amount> [--cadence weekly|monthly|yearly|custom|by-date] [--days N] [--date YYYY-MM-DD]`
- `envelope target list` - List all targets with suggested amounts
- `envelope target show <category>` - Show target details and suggested amounts for upcoming periods
- `envelope target delete <category>` - Remove target from a category
- `envelope target auto-fill [--period YYYY-MM]` - Auto-fill all budgets from targets

Registered in `src/cli/mod.rs` and `src/main.rs`.

## Key Files Reference

| File | Purpose | Status |
|------|---------|--------|
| `src/models/target.rs` | Core BudgetTarget model | ✅ Complete |
| `src/storage/targets.rs` | TargetRepository for JSON persistence | ✅ Complete |
| `src/services/budget.rs` | BudgetService with target methods | ✅ Complete |
| `src/tui/dialogs/budget.rs` | Unified budget/target dialog | ✅ Complete |
| `src/tui/app.rs` | App state with budget_dialog_state | ✅ Complete |
| `src/tui/handler.rs` | t/b/Enter key bindings for dialog | ✅ Complete |
| `src/tui/views/budget.rs` | Target indicators in budget view | ✅ Complete |
| `src/cli/target.rs` | CLI commands for targets | ✅ Complete |
| `src/cli/mod.rs` | CLI module exports | ✅ Complete |

## How Targets Work

1. **Setting a Target**: User presses 't', 'b', or Enter on a category in budget view, then switches to Target tab with Tab key
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

# Run all tests (324 should pass)
cargo test

# Run specific target tests
cargo test target
```

## Verification Summary

**Build Status:** ✅ `cargo check` passes
**Test Status:** ✅ All 324 tests pass (`cargo test`)

### Features Verified:
1. ✅ Core model with all cadence types and calculation logic
2. ✅ JSON persistence in `targets.json`
3. ✅ Budget service methods (set, get, delete, auto-fill)
4. ✅ TUI unified dialog with Period/Target tabs
5. ✅ Target indicators (◉) in budget view
6. ✅ Progress display for ByDate goals
7. ✅ CLI commands (set, list, show, delete, auto-fill)

## Technical Notes

- The Money type doesn't implement Mul/Div traits - calculations use `Money::from_cents()`
- The unified dialog approach was chosen over separate dialogs for better UX
- Target indicators show progress toward ByDate goals as percentage
