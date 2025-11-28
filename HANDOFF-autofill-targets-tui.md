# Handoff: Add Auto-Fill Targets to TUI Command Palette

## Context

The `auto_fill_all_targets()` function exists in `BudgetService` and is exposed via CLI (`envelope target auto-fill`), but is **NOT** available in the TUI command palette.

Users expect to be able to auto-fill their budget from targets directly in the TUI without dropping to the CLI.

## Key Files

| File | Purpose |
|------|---------|
| `src/tui/commands.rs` | Command definitions & `CommandAction` enum |
| `src/tui/handler.rs` | Command execution logic |
| `src/services/budget.rs:618-632` | `auto_fill_all_targets()` implementation |
| `src/cli/target.rs:219-261` | CLI implementation (reference for messaging) |

## Tasks

- [ ] **1. Add enum variant** - Add `AutoFillTargets` to `CommandAction` enum in `src/tui/commands.rs` (~line 55)

- [ ] **2. Add command entry** - Add to `COMMANDS` array in `src/tui/commands.rs` (~line 58):
  ```rust
  Command {
      name: "auto-fill-targets",
      description: "Fill budgets from targets",
      shortcut: None,
      action: CommandAction::AutoFillTargets,
  },
  ```

- [ ] **3. Add handler** - In `src/tui/handler.rs`, match on `CommandAction::AutoFillTargets` in the `execute_command` function and call the service

- [ ] **4. Show result** - Display notification with count of categories filled (see CLI implementation for messaging pattern)

- [ ] **5. Test** - Open TUI, press `:` for command palette, type "auto", verify command appears and executes correctly

## Reference Implementation (from CLI)

From `src/cli/target.rs:219-261`:

```rust
let budget_service = BudgetService::new(storage);
let allocations = budget_service.auto_fill_all_targets(&period)?;

if allocations.is_empty() {
    // No targets to auto-fill
} else {
    // Show success: "{} category/categories updated"
    // Show Available to Budget status
}
```

## Service Method Signature

From `src/services/budget.rs:618-632`:

```rust
/// Auto-fill budgets for all categories with targets
pub fn auto_fill_all_targets(
    &self,
    period: &BudgetPeriod,
) -> EnvelopeResult<Vec<BudgetAllocation>>
```

## Notes

- The current period should come from `app.current_period` in the TUI
- Use the notification system to show success/failure messages
- Consider also adding `AutoFillTarget` (singular) for filling just the selected category
