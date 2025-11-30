# Handoff: ByDate Target Progress - Implementation Complete

## Summary

Fixed the ByDate target progress calculation and added a preview feature. The progress now correctly tracks actual payments, and shows a preview of potential progress if budgeted amounts are paid.

## Changes Made

### 1. New Method: `calculate_cumulative_budgeted`

**File:** `src/services/budget.rs:414-428`

Calculates the sum of all budgeted amounts for a category across all periods up to a target date.

```rust
pub fn calculate_cumulative_budgeted(
    &self,
    category_id: CategoryId,
    up_to_period: &BudgetPeriod,
) -> EnvelopeResult<Money>
```

### 2. New Method: `calculate_cumulative_paid`

**File:** `src/services/budget.rs:436-464`

Calculates the sum of all payments (negative activity) for a category across all time up to a target date. Returns the absolute value of outflows.

```rust
pub fn calculate_cumulative_paid(
    &self,
    category_id: CategoryId,
    up_to_period: &BudgetPeriod,
) -> EnvelopeResult<Money>
```

### 3. Updated TUI Progress Display

**File:** `src/tui/views/budget.rs:223-275`

**Progress Logic:**
- **Paid is the source of truth** - If any payments exist, use cumulative paid
- **Budgeted is fallback only** - Only used when no payments have been made yet

**Preview Feature:**
- Shows potential progress if all budgeted money is paid
- Only displays unpaid budgeted amount (avoids double-counting)
- Format: `$2000 by Dec 2026 (5% → 10%)` where:
  - `5%` (magenta) = actual progress from payments
  - `→ 10%` (white) = preview if budgeted amount is also paid
- Arrow only appears when preview differs from progress by more than 0.5%

**Key Formula:**
```rust
// Progress: paid wins, budgeted is fallback
let progress_amount = if cumulative_paid.cents() > 0 {
    cumulative_paid.cents()
} else {
    cumulative_budgeted.cents().max(0)
};

// Preview: only add unpaid portion of budgeted
let unpaid_budgeted = (cumulative_budgeted.cents() - cumulative_paid.cents()).max(0);
let preview_amount = cumulative_paid.cents() + unpaid_budgeted;
```

### 4. New Imports

**File:** `src/tui/views/budget.rs:5,14`

```rust
use chrono::Datelike;
use crate::models::{AccountType, BudgetPeriod, TargetCadence};
```

### 5. New Tests

**File:** `src/services/budget.rs`

- `test_cumulative_budgeted_for_bydate_progress` (line 1033) - Tests cumulative budgeted calculation
- `test_cumulative_paid_for_bydate_progress` (line 1065) - Tests cumulative paid calculation
- `test_paid_wins_over_budgeted` (line 1117) - Verifies paid always wins when payments exist

## Behavior Examples

| Scenario | Budgeted | Paid | Progress | Preview |
|----------|----------|------|----------|---------|
| No activity | $0 | $0 | 0% | (none) |
| Budgeted only | $200 | $0 | 10% | (none) |
| Paid only | $0 | $100 | 5% | (none) |
| Paid = Budgeted | $100 | $100 | 5% | (none) |
| Paid < Budgeted | $200 | $100 | 5% | → 10% |
| Paid > Budgeted | $100 | $200 | 10% | (none) |

*Assuming $2000 target*

## Files Modified

1. `src/services/budget.rs` - Added two new methods and three tests
2. `src/tui/views/budget.rs` - Updated progress display with styled spans

## Test Results

- All 345 tests pass
- Release build compiles successfully

## Key Design Decisions

1. **Paid wins over budgeted** - The actual payment is the source of truth for debt payoff progress, not the intention to pay (budget)

2. **Budgeted as fallback** - Before any payments, budgeted shows planned progress

3. **Preview shows potential** - White preview percentage shows what progress would be if you follow through on your budget

4. **No double-counting** - Preview only adds unpaid budgeted amount, not total budgeted

## Remaining Work

### Fix: Suggested Budget Should Account for Cumulative Paid

**Problem:** The suggested budget calculation for ByDate targets does NOT consider what's already been paid. It divides the full target by months remaining, ignoring progress.

**Current behavior (wrong):**
- Target: $2000 by Dec 2026
- Already paid: $500
- Months remaining: 12
- Suggested: $2000 / 12 = **$167/month** (ignores the $500)

**Expected behavior:**
- Target: $2000 by Dec 2026
- Already paid: $500
- Remaining needed: $1500
- Months remaining: 12
- Suggested: $1500 / 12 = **$125/month**

**Location:** `src/models/target.rs:215-233` - `calculate_by_date_for_period` method

**The issue:** The model layer doesn't have access to storage/services to calculate cumulative paid.

**Solution options:**

1. **Move calculation to BudgetService** (recommended)
   - Create new method `get_suggested_budget_for_bydate` in `BudgetService`
   - This method can access `calculate_cumulative_paid`
   - Formula: `(target_amount - cumulative_paid) / months_remaining`

2. **Pass cumulative_paid into the model method**
   - Add optional parameter to `calculate_for_period`
   - Less clean but maintains current structure

**Implementation sketch (Option 1):**

```rust
// In src/services/budget.rs

/// Get suggested budget for a ByDate target, accounting for progress
pub fn get_suggested_budget_with_progress(
    &self,
    category_id: CategoryId,
    period: &BudgetPeriod,
) -> EnvelopeResult<Option<Money>> {
    let target = match self.storage.targets.get_for_category(category_id)? {
        Some(t) => t,
        None => return Ok(None),
    };

    match &target.cadence {
        TargetCadence::ByDate { target_date } => {
            let target_period = BudgetPeriod::monthly(target_date.year(), target_date.month());
            let cumulative_paid = self.calculate_cumulative_paid(category_id, &target_period)?;

            let remaining = (target.amount.cents() - cumulative_paid.cents()).max(0);
            let months = months_between(period.start_date(), *target_date);

            if months <= 0 {
                Ok(Some(Money::from_cents(remaining)))
            } else {
                Ok(Some(Money::from_cents((remaining as f64 / months as f64).ceil() as i64)))
            }
        }
        _ => Ok(Some(target.calculate_for_period(period))),
    }
}
```

**Files to modify:**
- `src/services/budget.rs` - Add new method
- Anywhere `get_suggested_budget` is called for ByDate targets - use new method instead

## Manual Test Scenarios

1. Create a category with ByDate target ($2000 by Dec 2026)
2. Budget $200, no payment → Should show 10% (budgeted fallback)
3. Pay $100 with $0 budgeted → Should show 5% (paid, no preview)
4. Pay $100, budget $200 → Should show 5% → 15% (paid + preview of unpaid $200)
5. Pay $200, budget $100 → Should show 10% (paid wins, no preview since budgeted < paid)
