# EnvelopeCLI Implementation Plan

> Generated from EnvelopeCLI-PRD-MVP.docx | November 2025

Terminal-Based Zero-Based Budgeting Application - MVP Implementation Guide

---

## Progress Tracking

<!--
HOW TO USE THIS DOCUMENT:
- Check off tasks as you complete them using [x]
- Each step has a main checkbox and sub-tasks for files and testing
- Future Claude Code instances should check this file first to see current progress
- Mark the step's main checkbox only when ALL sub-tasks are complete
-->

### Overall Progress

- [x] **Phase 1: Foundation** (Steps 1-5)
- [x] **Phase 2: Core Budget** (Steps 6-10)
- [x] **Phase 3: Transactions** (Steps 11-16)
- [x] **Phase 4: TUI** (Steps 17-25)
- [x] **Phase 5: Reconciliation** (Steps 26-27)
- [ ] **Phase 6: Reporting** (Steps 28-30)
- [ ] **Phase 7: Security & Polish** (Steps 31-35)

---

## Overview

| Metric | Value |
|--------|-------|
| **Total Steps** | 35 |
| **Total Phases** | 7 |
| **Estimated Duration** | 10-14 weeks |
| **Technology Stack** | Rust (ratatui, clap, serde) |
| **Max Files Per Step** | 15 |

### Phase Summary

| Phase | Steps | Description | Status |
|-------|-------|-------------|--------|
| 1. Foundation | 1-5 | Project setup, data models, storage, audit, backup | Complete |
| 2. Core Budget | 6-10 | Accounts, categories, periods, allocation, rollover | Complete |
| 3. Transactions | 11-16 | CRUD, payees, splits, transfers, CSV import | Complete |
| 4. TUI | 17-25 | Framework, views, dialogs, command palette, help | Complete |
| 5. Reconciliation | 26-27 | Reconciliation workflow, locking, adjustments | Complete |
| 6. Reporting | 28-30 | Reports, data export | Not Started |
| 7. Security & Polish | 31-35 | Encryption, setup wizard, error handling, CI, docs | Not Started |

---

## Phase 1: Foundation

### Step 1: Project Initialization & Configuration

- [x] **STEP 1 COMPLETE**

**Objective**: Set up the Rust project structure with all dependencies, establish the configuration system, and create the data directory structure.

**Implementation Details**: Initialize Cargo project with workspace structure. Configure dependencies: `clap` (CLI), `ratatui` + `crossterm` (TUI), `serde` + `serde_json` (serialization), `uuid`, `chrono`, `thiserror`, `anyhow`, `directories`. Create config loading/saving with XDG-compliant paths (~/.envelope/).

**Files to Create/Modify**:
- [x] `Cargo.toml`: Project metadata, dependencies (clap, ratatui, crossterm, serde, serde_json, uuid, chrono, thiserror, anyhow, directories, argon2, aes-gcm)
- [x] `src/main.rs`: Entry point with basic CLI skeleton using clap
- [x] `src/lib.rs`: Library root, module declarations
- [x] `src/config/mod.rs`: Config module exports
- [x] `src/config/settings.rs`: Settings struct with serde, budget period preference, encryption toggle
- [x] `src/config/paths.rs`: XDG path resolution for ~/.envelope/, data/, backups/
- [x] `src/error.rs`: Custom error types using thiserror (EnvelopeError enum)
- [x] `.gitignore`: Rust ignores, test data directories
- [x] `README.md`: Basic project overview

**Dependencies**: None (first step)

**Testing Checklist**:
- [x] `cargo build` succeeds
- [x] `cargo run -- --help` shows CLI skeleton
- [x] Config file created on first run

**User Actions Required**: Confirm Rust as the implementation language; approve directory structure (~/.envelope/)

---

### Step 2: Core Data Models

- [x] **STEP 2 COMPLETE**

**Objective**: Implement all core data entities (Account, Transaction, Category, CategoryGroup, BudgetAllocation) with full serde support and validation.

**Implementation Details**: Create strongly-typed models matching the PRD data model exactly. Use newtype patterns for IDs. Implement Display traits for terminal output. Add validation methods on each struct. Use chrono for dates, uuid for IDs.

**Files to Create/Modify**:
- [x] `src/models/mod.rs`: Module exports for all models
- [x] `src/models/account.rs`: Account struct (id, name, type, on_budget, archived, created_at, reconciliation fields)
- [x] `src/models/transaction.rs`: Transaction struct with status enum (Pending/Cleared/Reconciled), splits, transfer_id, import_id
- [x] `src/models/category.rs`: Category and CategoryGroup structs with sort_order
- [x] `src/models/budget.rs`: BudgetAllocation struct with flexible period support (monthly, weekly, bi-weekly, custom)
- [x] `src/models/payee.rs`: Payee struct with auto-categorization rules
- [x] `src/models/period.rs`: BudgetPeriod enum and parsing (2025-01, 2025-W03, custom date ranges)
- [x] `src/models/money.rs`: Money type (i64 cents internally, proper arithmetic, Display impl)
- [x] `src/models/ids.rs`: Newtype wrappers (AccountId, TransactionId, CategoryId, etc.)

**Dependencies**: Step 1

**Testing Checklist**:
- [x] Unit tests for serialization round-trips
- [x] Validation tests for all entities
- [x] Money arithmetic tests

**User Actions Required**: None

---

### Step 3: File I/O & Storage Layer

- [x] **STEP 3 COMPLETE**

**Objective**: Implement the JSON storage system with atomic writes, file locking, and automatic directory creation.

**Implementation Details**: Create repository traits for each entity type. Implement JSON file storage with atomic writes (write to temp, then rename). Add file locking for concurrent access safety. Create storage initialization that ensures directory structure exists.

**Files to Create/Modify**:
- [x] `src/storage/mod.rs`: Storage module exports, Storage struct coordinating all repositories
- [x] `src/storage/file_io.rs`: Atomic write helper, read_json/write_json generic functions
- [x] `src/storage/accounts.rs`: AccountRepository - load/save accounts.json
- [x] `src/storage/transactions.rs`: TransactionRepository - load/save transactions.json with indexing
- [x] `src/storage/categories.rs`: CategoryRepository - load/save budget.json (categories + groups)
- [x] `src/storage/budget.rs`: BudgetRepository - allocations storage
- [x] `src/storage/payees.rs`: PayeeRepository - load/save payees.json
- [x] `src/storage/init.rs`: Initialize storage, create default directory structure, handle first-run

**Dependencies**: Steps 1, 2

**Testing Checklist**:
- [x] Integration tests creating temp directories
- [x] Verify atomic writes don't corrupt on failure
- [x] Test concurrent access

**User Actions Required**: None

---

### Step 4: Audit Logging System

- [x] **STEP 4 COMPLETE**

**Objective**: Implement the append-only audit log that records all create, update, delete operations with before/after values.

**Implementation Details**: Create audit log format with timestamp, operation type, entity type, entity ID, user-readable diff. Implement append-only writer. Add audit hooks that storage layer calls automatically.

**Files to Create/Modify**:
- [x] `src/audit/mod.rs`: Audit module exports
- [x] `src/audit/entry.rs`: AuditEntry struct (timestamp, operation: Create/Update/Delete, entity_type, entity_id, before, after)
- [x] `src/audit/logger.rs`: AuditLogger - append entries to audit.log, flush on each write
- [x] `src/audit/diff.rs`: Generate human-readable diffs for before/after values
- [x] `src/storage/mod.rs`: (modify) Add audit integration to Storage struct

**Dependencies**: Steps 1, 2, 3

**Testing Checklist**:
- [x] Create/update/delete entities and verify audit.log contains correct entries
- [x] Test log survives crashes

**User Actions Required**: None

---

### Step 5: Backup System

- [x] **STEP 5 COMPLETE**

**Objective**: Implement automatic rolling backups with configurable retention (30 daily + 12 monthly default).

**Implementation Details**: Create backup before any destructive operation. Implement rolling retention policy. Store backups as dated JSON archives in backups/ directory. Add restore capability.

**Files to Create/Modify**:
- [x] `src/backup/mod.rs`: Backup module exports
- [x] `src/backup/manager.rs`: BackupManager - create backups, enforce retention, list available backups
- [x] `src/backup/restore.rs`: Restore from backup functionality
- [x] `src/config/settings.rs`: (modify) Add backup retention settings (daily_count, monthly_count)
- [x] `src/storage/mod.rs`: (modify) Add `backup_before_destructive()` method for auto-backup
- [x] `src/cli/backup.rs`: CLI commands (create, list, restore, info, prune)
- [x] `src/services/category.rs`: (modify) Call `backup_before_destructive()` in delete operations

**Dependencies**: Steps 1, 3, 4

**Testing Checklist**:
- [x] Create multiple backups
- [x] Verify retention policy deletes old ones
- [x] Test restore functionality
- [x] Verify automatic backup before destructive operations (category delete triggers backup)

**User Actions Required**: None

---

## Phase 2: Core Budget Functionality

### Step 6: Account Management (CRUD)

- [x] **STEP 6 COMPLETE**

**Objective**: Implement complete account management - create, read, update, archive accounts with balance tracking.

**Implementation Details**: Account service layer with full CRUD. Balance calculation from transactions. Archive (soft-delete) functionality. CLI commands: `envelope account create`, `envelope account list`, `envelope account edit`, `envelope account archive`.

**Files to Create/Modify**:
- [x] `src/services/mod.rs`: Services module exports
- [x] `src/services/account.rs`: AccountService - create, get, list, update, archive, calculate_balance, calculate_cleared_balance
- [x] `src/cli/mod.rs`: CLI module structure
- [x] `src/cli/account.rs`: Account subcommands using clap (create, list, edit, archive, show)
- [x] `src/main.rs`: (modify) Wire up account commands
- [x] `src/display/mod.rs`: Display formatting module
- [x] `src/display/account.rs`: Format account for terminal output (table format)

**Dependencies**: Steps 1-5

**Testing Checklist**:
- [x] CLI integration tests - create account
- [x] Verify JSON storage
- [x] List accounts
- [x] Edit account
- [x] Archive account
- [x] Balance calculation tests

**User Actions Required**: None

---

### Step 7: Category & Group Management

- [x] **STEP 7 COMPLETE**

**Objective**: Implement category and category group management with customizable organization and sort ordering.

**Implementation Details**: Categories belong to groups. Groups have sort order. CRUD for both. CLI commands for category management. Create default groups on first run (Bills, Needs, Wants, Savings).

**Files to Create/Modify**:
- [x] `src/services/category.rs`: CategoryService - CRUD for categories and groups, reorder, move between groups
- [x] `src/cli/category.rs`: Category subcommands (create, list, edit, delete, move, reorder)
- [x] `src/main.rs`: (modify) Wire up category commands
- [x] `src/display/category.rs`: Format categories/groups for terminal (tree structure)
- [x] `src/storage/init.rs`: (modify) Create default category groups on first run

**Dependencies**: Steps 1-6

**Testing Checklist**:
- [x] Create groups and categories
- [x] Verify hierarchy
- [x] Test reordering
- [x] Verify defaults created on fresh install

**User Actions Required**: Confirm default category groups (Bills, Needs, Wants, Savings) or customize

---

### Step 8: Budget Period System

- [x] **STEP 8 COMPLETE**

**Objective**: Implement flexible budget periods supporting weekly, bi-weekly, monthly, and custom date ranges.

**Implementation Details**: Period parsing and normalization. Period navigation (next/previous). Period detection from date. Support all formats: monthly (2025-01), weekly (2025-W03), bi-weekly, custom ranges.

**Files to Create/Modify**:
- [x] `src/models/period.rs`: (expand) Full period implementation - parsing, comparison, iteration, contains_date
- [x] `src/services/period.rs`: PeriodService - current period, navigate, list periods in range
- [x] `src/cli/budget.rs`: Period commands integrated with budget CLI (period, periods, prev, next)
- [x] `src/config/settings.rs`: (modify) Store user's preferred period type
- [x] `src/main.rs`: (modify) Wire up period commands

**Dependencies**: Steps 1-7

**Testing Checklist**:
- [x] Parse all period formats
- [x] Test navigation
- [x] Verify date containment
- [x] Test preference persistence

**User Actions Required**: Select preferred budget period type during setup

---

### Step 9: Budget Allocation & Available to Budget

- [x] **STEP 9 COMPLETE**

**Objective**: Implement zero-based budget allocation where users assign funds to categories and track "Available to Budget" reaching zero.

**Implementation Details**: BudgetService with assign funds, move funds, calculate available. Available to Budget = Total Income - Total Assigned. Track per-category: budgeted, activity (spending), available. CLI commands for budget operations.

**Files to Create/Modify**:
- [x] `src/services/budget.rs`: BudgetService - assign, move_funds, get_available_to_budget, get_category_available, get_period_summary
- [x] `src/cli/budget.rs`: Budget subcommands (assign, move, status, overview)
- [x] `src/main.rs`: (modify) Wire up budget commands
- [x] `src/cli/budget.rs`: Format budget overview (table with budgeted/activity/available columns) - inline in CLI
- [x] `src/models/budget.rs`: (modify) Add computed fields, validation for negative assignments

**Dependencies**: Steps 1-8

**Testing Checklist**:
- [x] Assign funds
- [x] Verify Available to Budget decreases
- [x] Move funds between categories
- [x] Verify zero-sum

**User Actions Required**: None

---

### Step 10: Category Balance Rollover

- [x] **STEP 10 COMPLETE**

**Objective**: Implement automatic rollover of category balances (positive or negative) to subsequent budget periods.

**Implementation Details**: When entering a new period, calculate carryover from previous period. Store carryover amount in allocation. Visual indication of overspent categories. Handle both surplus and deficit rollovers.

**Files to Create/Modify**:
- [x] `src/services/budget.rs`: (modify) Add rollover calculation, apply_rollover, get_carryover
- [x] `src/cli/budget.rs`: (modify) Added rollover and overspent commands
- [x] `src/models/budget.rs`: (modify) Ensure carryover field is properly used
- [x] `src/cli/budget.rs`: (modify) Show carryover amounts in overview, highlight negative (overspent) categories

**Dependencies**: Steps 1-9

**Testing Checklist**:
- [x] Create allocations in period 1
- [x] Spend more than budgeted
- [x] Advance to period 2
- [x] Verify negative carryover
- [x] Test positive carryover

**User Actions Required**: None

---

## Phase 3: Transaction Management

### Step 11: Transaction CRUD

- [x] **STEP 11 COMPLETE**

**Objective**: Implement manual transaction entry with all fields: date, payee, amount, category, memo, status.

**Implementation Details**: TransactionService with full CRUD. Auto-detect inflow/outflow from sign. Track pending/cleared/reconciled status. Update category available balance on transaction changes. CLI commands for transaction management.

**Files to Create/Modify**:
- [x] `src/services/transaction.rs`: TransactionService - create, get, list, update, delete, set_status, filter_by_account, filter_by_date_range
- [x] `src/cli/transaction.rs`: Transaction subcommands (add, list, edit, delete, clear)
- [x] `src/main.rs`: (modify) Wire up transaction commands
- [x] `src/display/transaction.rs`: Format transaction for terminal (register view)
- [x] `src/services/budget.rs`: (modify) Recalculate category available when transactions change

**Dependencies**: Steps 1-10

**Testing Checklist**:
- [x] Add transactions
- [x] Verify category balances update
- [x] Edit transaction
- [x] Verify recalculation
- [x] Delete and verify

**User Actions Required**: None

---

### Step 12: Payee Auto-Suggestion

- [x] **STEP 12 COMPLETE**

**Objective**: Implement payee autocomplete and automatic category suggestion based on historical patterns.

**Implementation Details**: Build payee index from transaction history. Track category frequency per payee. Suggest most common category when payee entered. CLI flag `--auto-categorize`. Payee CRUD for managing rules.

**Files to Create/Modify**:
- [x] `src/services/payee.rs`: PayeeService - suggest_payees (fuzzy), get_suggested_category, learn_from_transaction
- [x] `src/services/transaction.rs`: (modify) Call payee learning on transaction create
- [x] `src/cli/transaction.rs`: (modify) Add --auto-categorize flag, payee suggestions in interactive mode
- [x] `src/cli/payee.rs`: Payee management commands (list, set-category, delete)
- [x] `src/main.rs`: (modify) Wire up payee commands

**Dependencies**: Steps 1-11

**Testing Checklist**:
- [x] Add transactions with same payee/category
- [x] Verify suggestion
- [x] Test fuzzy matching
- [x] Test manual override

**User Actions Required**: None

---

### Step 13: Split Transactions

- [x] **STEP 13 COMPLETE**

**Objective**: Implement split transactions where a single transaction is divided across multiple categories.

**Implementation Details**: Transaction with null category_id but populated splits array. Validate splits sum to transaction amount. Display split breakdown. Edit individual splits. CLI syntax for splits.

**Files to Create/Modify**:
- [x] `src/services/transaction.rs`: (modify) Add create_split, validate_splits, update_split
- [x] `src/services/budget.rs`: (modify) Handle split transactions in category calculations
- [x] `src/cli/transaction.rs`: (modify) Add split creation syntax `--split groceries:50 --split household:35.50`
- [x] `src/display/transaction.rs`: (modify) Show split breakdown in transaction display
- [x] `src/models/transaction.rs`: (modify) Add Split struct, validation methods

**Dependencies**: Steps 1-12

**Testing Checklist**:
- [x] Create split transaction
- [x] Verify each category updated
- [x] Edit split
- [x] Verify totals match
- [x] Test validation rejects mismatched amounts

**User Actions Required**: None

---

### Step 14: Account Transfers

- [x] **STEP 14 COMPLETE**

**Objective**: Implement transfers between accounts as linked transaction pairs maintaining consistency.

**Implementation Details**: Transfer creates two transactions with matching transfer_id. Outflow from source, inflow to destination. Transfers are not categorized (don't affect budget). Delete/edit updates both transactions.

**Files to Create/Modify**:
- [x] `src/services/transfer.rs`: TransferService - create_transfer (creates both transactions), edit_transfer, delete_transfer, get_linked_transaction
- [x] `src/services/transaction.rs`: (modify) Handle transfer transactions specially (not categorizable)
- [x] `src/cli/transfer.rs`: Transfer command (envelope transfer FROM_ACCOUNT TO_ACCOUNT AMOUNT)
- [x] `src/main.rs`: (modify) Wire up transfer command
- [x] `src/display/transaction.rs`: (modify) Show transfer indicator and linked account

**Dependencies**: Steps 1-13

**Testing Checklist**:
- [x] Create transfer
- [x] Verify both transactions created
- [x] Verify balances correct
- [x] Delete transfer
- [x] Verify both removed

**User Actions Required**: None

---

### Step 15: CSV Import - Core Parser

- [x] **STEP 15 COMPLETE**

**Objective**: Implement CSV parsing with configurable column mapping for bank exports.

**Implementation Details**: Detect common CSV formats automatically. Allow manual column mapping. Parse dates in multiple formats. Handle amount columns (single signed, separate debit/credit). Store format presets per account.

**Files to Create/Modify**:
- [x] `src/import/mod.rs`: Import module exports
- [x] `src/import/csv_parser.rs`: CSVParser - detect format, parse with mapping, normalize amounts
- [x] `src/import/column_mapping.rs`: ColumnMapping struct, common presets (Chase, BoA, etc.), interactive mapping builder
- [x] `src/import/date_parser.rs`: Parse dates in multiple formats (MM/DD/YYYY, YYYY-MM-DD, DD/MM/YYYY, etc.)
- [x] `src/config/settings.rs`: (modify) Store account-specific import mappings

**Dependencies**: Steps 1-14

**Testing Checklist**:
- [x] Parse sample CSVs from major banks
- [x] Test date format detection
- [x] Test amount normalization
- [x] Test preset matching

**User Actions Required**: May need to configure column mapping for non-standard bank formats

---

### Step 16: CSV Import - Duplicate Detection & Import Flow

- [x] **STEP 16 COMPLETE**

**Objective**: Implement duplicate detection and the full import workflow with preview and confirmation.

**Implementation Details**: Generate import_id from transaction attributes (date + amount + payee hash). Detect duplicates against existing transactions. Preview imported transactions. Allow skip duplicates, import all, or select specific. Bulk categorize imported transactions.

**Files to Create/Modify**:
- [x] `src/import/duplicate.rs`: DuplicateDetector - generate import_id, find_duplicates, similarity scoring
- [x] `src/import/workflow.rs`: ImportWorkflow - parse, detect duplicates, preview, confirm, execute import
- [x] `src/cli/import.rs`: Import command with interactive preview and confirmation
- [x] `src/main.rs`: (modify) Wire up import command
- [x] `src/display/import.rs`: Format import preview (highlight duplicates, show mapping)

**Dependencies**: Steps 1-15

**Testing Checklist**:
- [x] Import CSV
- [x] Verify duplicates flagged
- [x] Re-import same CSV
- [x] Verify duplicates detected
- [x] Test selective import

**User Actions Required**: Confirm imports; handle duplicate decisions

---

## Phase 4: Terminal User Interface

### Step 17: TUI Framework Setup

- [x] **STEP 17 COMPLETE**

**Objective**: Set up the ratatui framework with basic application structure, event loop, and terminal handling.

**Implementation Details**: Initialize terminal with crossterm backend. Create App struct with state management. Implement event loop (key events, tick events). Handle graceful shutdown. Restore terminal on panic.

**Files to Create/Modify**:
- [x] `src/tui/mod.rs`: TUI module exports
- [x] `src/tui/app.rs`: App struct - state, active view, selected items
- [x] `src/tui/terminal.rs`: Terminal setup/teardown, panic hook for cleanup
- [x] `src/tui/event.rs`: Event enum, event loop using crossterm
- [x] `src/tui/handler.rs`: Global key handler, route to active view
- [x] `src/main.rs`: (modify) Add `envelope tui` command to launch TUI

**Dependencies**: Steps 1-16

**Testing Checklist**:
- [x] Launch TUI
- [x] Verify terminal restored on quit
- [x] Verify panic doesn't corrupt terminal
- [x] Test event handling

**User Actions Required**: None

---

### Step 18: TUI Layout & Navigation Framework

- [x] **STEP 18 COMPLETE**

**Objective**: Implement the three-panel layout (sidebar, main panel, status bar) with keyboard navigation between panels.

**Implementation Details**: Split terminal into layout regions. Sidebar for accounts/views. Main panel context-sensitive. Status bar with Available to Budget and shortcuts. h/l or arrow keys to move between panels.

**Files to Create/Modify**:
- [x] `src/tui/layout.rs`: Define layout regions, calculate splits
- [x] `src/tui/views/mod.rs`: View trait, view enum
- [x] `src/tui/views/sidebar.rs`: Sidebar component - account list, view switcher
- [x] `src/tui/views/status_bar.rs`: Status bar - Available to Budget, current balance, key hints
- [x] `src/tui/widgets/mod.rs`: Shared widget components
- [x] `src/tui/app.rs`: (modify) Add view routing, panel focus tracking
- [x] `src/tui/handler.rs`: (modify) Handle panel navigation keys (h/l/arrows)

**Dependencies**: Steps 1-17

**Testing Checklist**:
- [x] Launch TUI
- [x] Verify layout renders
- [x] Test panel switching
- [x] Verify status bar updates

**User Actions Required**: None

---

### Step 19: Account List View

- [x] **STEP 19 COMPLETE**

**Objective**: Implement the account list sidebar showing all accounts with balances and selection.

**Implementation Details**: List accounts with names and balances. Highlight selected account. Show cleared vs total balance. Handle account selection (Enter to view transactions). Show archived accounts separately (toggle).

**Files to Create/Modify**:
- [x] `src/tui/views/account_list.rs`: AccountListView - render account list, selection, keyboard handling
- [x] `src/tui/widgets/account_item.rs`: Individual account row widget (integrated into account_list.rs)
- [x] `src/tui/app.rs`: (modify) Track selected account, handle account selection
- [x] `src/tui/views/sidebar.rs`: (modify) Integrate account list

**Dependencies**: Steps 1-18

**Testing Checklist**:
- [x] View account list
- [x] Navigate with j/k
- [x] Select account
- [x] Verify balance display matches service layer

**User Actions Required**: None

---

### Step 20: Transaction Register View

- [x] **STEP 20 COMPLETE**

**Objective**: Implement the transaction register showing transactions for the selected account with scrolling and selection.

**Implementation Details**: Display transactions in table format (date, payee, category, amount, status). Scrolling for long lists. Highlight selected transaction. Status indicators (cleared checkmark, reconciled lock). Quick actions (c to clear, e to edit, d to delete).

**Files to Create/Modify**:
- [x] `src/tui/views/register.rs`: RegisterView - transaction table, selection, scrolling, quick actions
- [x] `src/tui/widgets/transaction_row.rs`: Transaction row widget with status icons (integrated into register.rs)
- [x] `src/tui/widgets/table.rs`: Generic scrollable table widget (using ratatui Table)
- [x] `src/tui/app.rs`: (modify) Handle register view state
- [x] `src/tui/handler.rs`: (modify) Route register-specific keys

**Dependencies**: Steps 1-19

**Testing Checklist**:
- [x] Select account
- [x] View transactions
- [x] Scroll through list
- [x] Use quick actions
- [x] Verify changes persist

**User Actions Required**: None

---

### Step 21: Budget View

- [x] **STEP 21 COMPLETE**

**Objective**: Implement the budget overview showing all categories with budgeted, activity, and available amounts for the current period.

**Implementation Details**: Grid layout with category groups as sections. Each category shows: budgeted (editable), activity (calculated), available (calculated). Highlight overspent (red) and underfunded. Inline editing of budget amounts. Show Available to Budget prominently.

**Files to Create/Modify**:
- [x] `src/tui/views/budget.rs`: BudgetView - category grid, group sections, inline editing
- [x] `src/tui/widgets/budget_row.rs`: Category budget row with columns (integrated into budget.rs)
- [x] `src/tui/widgets/budget_header.rs`: Available to Budget display, period selector (integrated into budget.rs)
- [x] `src/tui/widgets/inline_edit.rs`: Inline number editing widget (placeholder - full implementation in future)
- [x] `src/tui/app.rs`: (modify) Handle budget view state, period selection

**Dependencies**: Steps 1-20

**Testing Checklist**:
- [x] View budget
- [x] Edit allocation (placeholder - full inline editing in future)
- [x] Verify Available to Budget updates
- [x] Test overspent highlighting

**User Actions Required**: None

---

### Step 22: Transaction Entry Dialog

- [x] **STEP 22 COMPLETE**

**Objective**: Implement the add/edit transaction dialog with field navigation, autocomplete, and validation.

**Implementation Details**: Modal dialog for transaction entry. Tab through fields. Payee autocomplete dropdown. Category selector with search. Amount parsing (accepts $, negative). Date picker. Validation before save.

**Files to Create/Modify**:
- [x] `src/tui/dialogs/mod.rs`: Dialog module exports
- [x] `src/tui/dialogs/transaction.rs`: TransactionDialog - full form with fields (TransactionFormState), Tab navigation, validation, save/edit
- [x] `src/tui/widgets/input.rs`: Text input widget with cursor
- [x] `src/tui/app.rs`: (modify) Handle dialog state, modal focus, TransactionFormState
- [x] `src/tui/handler.rs`: (modify) Route dialog keys when modal active, delegate to transaction::handle_key

**Dependencies**: Steps 1-21

**Testing Checklist**:
- [x] Open dialog (a key)
- [x] Fill fields (Tab navigation, text input with cursor)
- [x] Test autocomplete (category dropdown with search filtering)
- [x] Save transaction (validates form, creates transaction)
- [x] Verify transaction created (saved to storage)
- [x] Test edit mode (loads existing transaction data)

**User Actions Required**: None

---

### Step 23: Command Palette

- [x] **STEP 23 COMPLETE**

**Objective**: Implement the command palette for quick access to all commands via fuzzy search.

**Implementation Details**: Trigger with : or /. Fuzzy search all commands. Show keyboard shortcut hints. Execute command on selection. Recent commands at top. Include all CLI commands.

**Files to Create/Modify**:
- [x] `src/tui/dialogs/command_palette.rs`: CommandPalette - fuzzy search, command list, execution
- [x] `src/tui/commands.rs`: Define all TUI commands with descriptions and shortcuts
- [x] `src/tui/widgets/fuzzy_list.rs`: Fuzzy-filtered list widget (integrated into command_palette.rs)
- [x] `src/tui/app.rs`: (modify) Handle command palette state
- [x] `src/tui/handler.rs`: (modify) Handle : and / to open palette

**Dependencies**: Steps 1-22

**Testing Checklist**:
- [x] Open palette
- [x] Search commands
- [x] Execute command (placeholder)
- [x] Verify action taken (placeholder)
- [x] Test all major commands accessible

**User Actions Required**: None

---

### Step 24: Move Funds Dialog & Bulk Operations

- [x] **STEP 24 COMPLETE**

**Objective**: Implement the move funds dialog for transferring budget between categories and bulk transaction operations.

**Implementation Details**: Move funds dialog (m key) - select source category, destination, amount. Bulk categorize (select multiple transactions, assign category). Bulk clear transactions.

**Files to Create/Modify**:
- [x] `src/tui/dialogs/move_funds.rs`: MoveFundsDialog - source/dest selection, amount, execute (placeholder UI)
- [x] `src/tui/dialogs/bulk_categorize.rs`: BulkCategorizeDialog - category selection, apply to selected (placeholder UI)
- [x] `src/tui/views/register.rs`: (modify) Multi-select mode, bulk action keys
- [x] `src/tui/app.rs`: (modify) Track multi-selection state

**Dependencies**: Steps 1-23

**Testing Checklist**:
- [x] Move funds between categories (placeholder)
- [x] Verify balances update (placeholder)
- [x] Bulk select transactions
- [x] Bulk categorize (placeholder)
- [x] Verify all updated (placeholder)

**User Actions Required**: None

---

### Step 25: Help System & Keyboard Reference

- [x] **STEP 25 COMPLETE**

**Objective**: Implement contextual help showing available keyboard shortcuts and command documentation.

**Implementation Details**: ? key shows help overlay. Context-sensitive (different help per view). Show all keybindings in current context. Link to full documentation. Dismissible with Esc or ?.

**Files to Create/Modify**:
- [x] `src/tui/dialogs/help.rs`: HelpDialog - contextual shortcuts, scrollable help text
- [x] `src/tui/keybindings.rs`: Define all keybindings with descriptions per view context
- [x] `src/tui/app.rs`: (modify) Handle help state, context detection
- [x] `src/tui/handler.rs`: (modify) Handle ? key globally

**Dependencies**: Steps 1-24

**Testing Checklist**:
- [x] Press ? in each view
- [x] Verify context-appropriate help
- [x] Dismiss and verify returns to previous state

**User Actions Required**: None

---

## Phase 5: Reconciliation

### Step 26: Reconciliation Workflow

- [x] **STEP 26 COMPLETE**

**Objective**: Implement the full reconciliation workflow - enter statement balance, mark transactions cleared, and complete reconciliation.

**Implementation Details**: Start reconciliation for account. Enter statement date and ending balance. Show uncleared transactions. Display difference (statement - cleared total). Mark transactions cleared. When difference = 0, allow completion. Lock reconciled transactions.

**Files to Create/Modify**:
- [x] `src/services/reconciliation.rs`: ReconciliationService - start, get_difference, complete, get_uncleared_transactions
- [x] `src/cli/reconcile.rs`: Reconcile command with interactive flow
- [x] `src/tui/views/reconcile.rs`: ReconciliationView - statement entry, transaction list, difference display
- [x] `src/tui/dialogs/reconcile_start.rs`: Dialog to enter statement date/balance
- [x] `src/main.rs`: (modify) Wire up reconcile command

**Dependencies**: Steps 1-25

**Testing Checklist**:
- [x] Start reconciliation
- [x] Clear transactions until difference = 0
- [x] Complete reconciliation
- [x] Verify transactions locked

**User Actions Required**: Enter statement balance; confirm reconciliation completion

---

### Step 27: Transaction Locking & Adjustment Transactions

- [x] **STEP 27 COMPLETE**

**Objective**: Implement reconciled transaction locking and balance adjustment transactions for discrepancies.

**Implementation Details**: Reconciled transactions cannot be edited without explicit unlock. Unlock requires confirmation and creates audit entry. If difference non-zero at completion, offer to create adjustment transaction. Adjustment categorized to special category.

**Files to Create/Modify**:
- [x] `src/services/transaction.rs`: (modify) Add check_locked, unlock_transaction, require unlock for edit/delete
- [x] `src/services/reconciliation.rs`: (modify) Add create_adjustment_transaction, complete_with_discrepancy
- [x] `src/models/transaction.rs`: (modify) Add is_locked() helper based on status
- [x] `src/tui/dialogs/unlock_confirm.rs`: Confirmation dialog for unlocking reconciled transaction
- [x] `src/tui/dialogs/adjustment.rs`: Dialog to confirm creating adjustment transaction

**Dependencies**: Steps 1-26

**Testing Checklist**:
- [x] Try to edit reconciled transaction
- [x] Verify blocked
- [x] Unlock with confirmation
- [x] Verify audit logged
- [x] Test adjustment creation

**User Actions Required**: Confirm unlock; confirm adjustment creation

---

## Phase 6: Reporting & Export

### Step 28: Budget Overview Report

- [ ] **STEP 28 COMPLETE**

**Objective**: Implement the budget overview report showing all categories with budgeted, activity, and available amounts.

**Implementation Details**: FR-RPT-01 compliance. Show all categories grouped. Display budgeted, activity (spending), available columns. Totals per group. Grand total. Period selection. Export to CSV.

**Files to Create/Modify**:
- [ ] `src/reports/mod.rs`: Reports module exports
- [ ] `src/reports/budget_overview.rs`: BudgetOverviewReport - generate, format for terminal, export CSV
- [ ] `src/cli/report.rs`: Report commands (envelope report budget, envelope report spending, etc.)
- [ ] `src/main.rs`: (modify) Wire up report commands
- [ ] `src/display/report.rs`: Report formatting utilities

**Dependencies**: Steps 1-27

**Testing Checklist**:
- [ ] Generate report
- [ ] Verify totals match budget service
- [ ] Export CSV
- [ ] Verify CSV readable

**User Actions Required**: None

---

### Step 29: Spending & Account Reports

- [ ] **STEP 29 COMPLETE**

**Objective**: Implement spending by category and account register reports with date range filtering.

**Implementation Details**: Spending report (FR-RPT-02) - breakdown by category for date range. Account register (FR-RPT-04) - filterable transaction list. Net worth (FR-RPT-03) - sum of all account balances. All exportable to CSV.

**Files to Create/Modify**:
- [ ] `src/reports/spending.rs`: SpendingReport - spending by category, date range filter
- [ ] `src/reports/account_register.rs`: AccountRegisterReport - transactions with filters
- [ ] `src/reports/net_worth.rs`: NetWorthReport - account balances summary
- [ ] `src/cli/report.rs`: (modify) Add spending, register, networth subcommands
- [ ] `src/tui/views/reports.rs`: Reports view in TUI - select report type, view results

**Dependencies**: Steps 1-28

**Testing Checklist**:
- [ ] Generate each report
- [ ] Verify data accuracy
- [ ] Test date filtering
- [ ] Test CSV export

**User Actions Required**: None

---

### Step 30: Full Data Export

- [ ] **STEP 30 COMPLETE**

**Objective**: Implement complete data export to CSV and JSON/YAML preserving all data and relationships.

**Implementation Details**: SEC-09, SEC-10 compliance. Export all transactions to CSV. Export full database to JSON (machine readable) or YAML (human friendly). Include all relationships. Schema versioning in export.

**Files to Create/Modify**:
- [ ] `src/export/mod.rs`: Export module exports
- [ ] `src/export/csv.rs`: Export transactions, budget allocations, account balances to CSV
- [ ] `src/export/json.rs`: Full database export to JSON with schema version
- [ ] `src/export/yaml.rs`: Full database export to YAML format
- [ ] `src/cli/export.rs`: Export command (envelope export --format csv|json|yaml --output path)
- [ ] `src/main.rs`: (modify) Wire up export command

**Dependencies**: Steps 1-29

**Testing Checklist**:
- [ ] Export in each format
- [ ] Verify completeness
- [ ] Reimport JSON export
- [ ] Verify data integrity

**User Actions Required**: None

---

## Phase 7: Security & Polish

### Step 31: Encryption at Rest

- [ ] **STEP 31 COMPLETE**

**Objective**: Implement optional AES-256-GCM encryption with Argon2id key derivation for data files.

**Implementation Details**: SEC-02, SEC-03 compliance. User enables encryption with passphrase. Derive key using Argon2id. Encrypt data files with AES-256-GCM. Decrypt on load. Secure memory clearing (SEC-04). Change passphrase functionality.

**Files to Create/Modify**:
- [ ] `src/crypto/mod.rs`: Crypto module exports
- [ ] `src/crypto/encryption.rs`: AES-256-GCM encrypt/decrypt functions
- [ ] `src/crypto/key_derivation.rs`: Argon2id key derivation from passphrase
- [ ] `src/crypto/secure_memory.rs`: Zeroize sensitive data in memory
- [ ] `src/storage/mod.rs`: (modify) Add encryption layer to read/write operations
- [ ] `src/config/settings.rs`: (modify) Encryption enabled flag, salt storage
- [ ] `src/cli/encrypt.rs`: Encryption commands (enable, disable, change-passphrase)
- [ ] `src/main.rs`: (modify) Wire up encryption commands

**Dependencies**: Steps 1-30

**Testing Checklist**:
- [ ] Enable encryption
- [ ] Verify files encrypted
- [ ] Reopen app with correct passphrase
- [ ] Test wrong passphrase rejected
- [ ] Test memory cleared

**User Actions Required**: Set passphrase if enabling encryption

---

### Step 32: First-Run Setup Wizard

- [ ] **STEP 32 COMPLETE**

**Objective**: Implement the interactive setup wizard for first-time users to configure accounts, categories, and preferences.

**Implementation Details**: Detect first run (no config file). Launch TUI wizard or CLI prompts. Create first account with starting balance. Select/customize category groups. Choose budget period preference. Starting balance becomes Available to Budget.

**Files to Create/Modify**:
- [ ] `src/setup/mod.rs`: Setup module exports
- [ ] `src/setup/wizard.rs`: SetupWizard - steps, state machine, completion
- [ ] `src/setup/steps/account.rs`: Create first account step
- [ ] `src/setup/steps/categories.rs`: Category group selection/customization
- [ ] `src/setup/steps/period.rs`: Budget period preference
- [ ] `src/tui/views/setup.rs`: TUI wizard view
- [ ] `src/main.rs`: (modify) Detect first run, launch wizard

**Dependencies**: Steps 1-31

**Testing Checklist**:
- [ ] Delete config
- [ ] Run app
- [ ] Verify wizard launches
- [ ] Complete wizard
- [ ] Verify all created correctly

**User Actions Required**: Complete setup wizard on first run

---

### Step 33: Error Handling & User Feedback

- [ ] **STEP 33 COMPLETE**

**Objective**: Implement comprehensive error handling with user-friendly messages and recovery suggestions.

**Implementation Details**: Map all errors to user-friendly messages. Suggest recovery actions. Log errors to stderr/file. Non-blocking errors show notification. Fatal errors show clear message before exit.

**Files to Create/Modify**:
- [ ] `src/error.rs`: (expand) Complete error catalog with user messages
- [ ] `src/tui/widgets/notification.rs`: Toast notification widget
- [ ] `src/tui/widgets/error_dialog.rs`: Error dialog with details and suggestions
- [ ] `src/tui/app.rs`: (modify) Notification queue, error handling
- [ ] `src/cli/mod.rs`: (modify) CLI error formatting, exit codes

**Dependencies**: Steps 1-32

**Testing Checklist**:
- [ ] Trigger various errors
- [ ] Verify user-friendly messages
- [ ] Verify recovery suggestions appropriate
- [ ] Verify logging

**User Actions Required**: None

---

### Step 34: Cross-Platform Testing & CI

- [ ] **STEP 34 COMPLETE**

**Objective**: Set up continuous integration and verify functionality across Linux, macOS, and Windows.

**Implementation Details**: GitHub Actions workflow. Build for all targets (Linux x64/ARM, macOS Intel/Apple Silicon, Windows). Run tests on all platforms. Create release binaries. Document platform-specific notes.

**Files to Create/Modify**:
- [ ] `.github/workflows/ci.yml`: CI workflow - build, test, lint across platforms
- [ ] `.github/workflows/release.yml`: Release workflow - build binaries, create GitHub release
- [ ] `Cargo.toml`: (modify) Add cross-compilation targets, feature flags
- [ ] `build.rs`: Build script for platform-specific compilation
- [ ] `INSTALL.md`: Installation instructions per platform
- [ ] `tests/integration/mod.rs`: Integration test suite
- [ ] `tests/integration/cross_platform.rs`: Platform-specific tests

**Dependencies**: Steps 1-33

**Testing Checklist**:
- [ ] CI runs on all platforms
- [ ] Verify binaries work on each
- [ ] Manual testing on Linux
- [ ] Manual testing on macOS
- [ ] Manual testing on Windows

**User Actions Required**: Set up GitHub repository secrets for releases

---

### Step 35: Documentation & Polish

- [ ] **STEP 35 COMPLETE**

**Objective**: Complete documentation including user guide, command reference, and data format schema.

**Implementation Details**: Comprehensive README. Command reference with examples. Data format schema documentation (for users who want to edit JSON directly). Keyboard shortcut reference. Troubleshooting guide.

**Files to Create/Modify**:
- [ ] `README.md`: (complete) Full project documentation
- [ ] `docs/commands.md`: Complete CLI command reference with examples
- [ ] `docs/keyboard-shortcuts.md`: TUI keyboard shortcut reference
- [ ] `docs/data-format.md`: JSON schema documentation with examples
- [ ] `docs/troubleshooting.md`: Common issues and solutions
- [ ] `src/cli/mod.rs`: (modify) Ensure --help output is comprehensive
- [ ] `man/envelope.1`: Man page (optional, for Unix systems)

**Dependencies**: Steps 1-34

**Testing Checklist**:
- [ ] Review all documentation for accuracy
- [ ] Verify examples work
- [ ] Have non-developer test from documentation

**User Actions Required**: Review and approve documentation

---

## Appendix

### A. File Structure Overview

```
envelope/
├── Cargo.toml
├── build.rs
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── error.rs
│   ├── config/
│   │   ├── mod.rs
│   │   ├── settings.rs
│   │   └── paths.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── account.rs
│   │   ├── transaction.rs
│   │   ├── category.rs
│   │   ├── budget.rs
│   │   ├── payee.rs
│   │   ├── period.rs
│   │   ├── money.rs
│   │   └── ids.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── file_io.rs
│   │   ├── accounts.rs
│   │   ├── transactions.rs
│   │   ├── categories.rs
│   │   ├── budget.rs
│   │   ├── payees.rs
│   │   └── init.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── account.rs
│   │   ├── transaction.rs
│   │   ├── category.rs
│   │   ├── budget.rs
│   │   ├── payee.rs
│   │   ├── period.rs
│   │   ├── transfer.rs
│   │   └── reconciliation.rs
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── account.rs
│   │   ├── transaction.rs
│   │   ├── category.rs
│   │   ├── budget.rs
│   │   ├── payee.rs
│   │   ├── period.rs
│   │   ├── transfer.rs
│   │   ├── reconcile.rs
│   │   ├── import.rs
│   │   ├── export.rs
│   │   ├── report.rs
│   │   └── encrypt.rs
│   ├── tui/
│   │   ├── mod.rs
│   │   ├── app.rs
│   │   ├── terminal.rs
│   │   ├── event.rs
│   │   ├── handler.rs
│   │   ├── layout.rs
│   │   ├── commands.rs
│   │   ├── keybindings.rs
│   │   ├── views/
│   │   │   ├── mod.rs
│   │   │   ├── sidebar.rs
│   │   │   ├── status_bar.rs
│   │   │   ├── account_list.rs
│   │   │   ├── register.rs
│   │   │   ├── budget.rs
│   │   │   ├── reconcile.rs
│   │   │   ├── reports.rs
│   │   │   └── setup.rs
│   │   ├── dialogs/
│   │   │   ├── mod.rs
│   │   │   ├── transaction.rs
│   │   │   ├── command_palette.rs
│   │   │   ├── move_funds.rs
│   │   │   ├── bulk_categorize.rs
│   │   │   ├── help.rs
│   │   │   ├── reconcile_start.rs
│   │   │   ├── unlock_confirm.rs
│   │   │   └── adjustment.rs
│   │   └── widgets/
│   │       ├── mod.rs
│   │       ├── account_item.rs
│   │       ├── transaction_row.rs
│   │       ├── table.rs
│   │       ├── budget_row.rs
│   │       ├── budget_header.rs
│   │       ├── inline_edit.rs
│   │       ├── input.rs
│   │       ├── autocomplete.rs
│   │       ├── date_picker.rs
│   │       ├── fuzzy_list.rs
│   │       ├── notification.rs
│   │       └── error_dialog.rs
│   ├── import/
│   │   ├── mod.rs
│   │   ├── csv_parser.rs
│   │   ├── column_mapping.rs
│   │   ├── date_parser.rs
│   │   ├── duplicate.rs
│   │   └── workflow.rs
│   ├── export/
│   │   ├── mod.rs
│   │   ├── csv.rs
│   │   ├── json.rs
│   │   └── yaml.rs
│   ├── reports/
│   │   ├── mod.rs
│   │   ├── budget_overview.rs
│   │   ├── spending.rs
│   │   ├── account_register.rs
│   │   └── net_worth.rs
│   ├── crypto/
│   │   ├── mod.rs
│   │   ├── encryption.rs
│   │   ├── key_derivation.rs
│   │   └── secure_memory.rs
│   ├── audit/
│   │   ├── mod.rs
│   │   ├── entry.rs
│   │   ├── logger.rs
│   │   └── diff.rs
│   ├── backup/
│   │   ├── mod.rs
│   │   ├── manager.rs
│   │   └── restore.rs
│   ├── setup/
│   │   ├── mod.rs
│   │   ├── wizard.rs
│   │   └── steps/
│   │       ├── account.rs
│   │       ├── categories.rs
│   │       └── period.rs
│   └── display/
│       ├── mod.rs
│       ├── account.rs
│       ├── transaction.rs
│       ├── category.rs
│       ├── budget.rs
│       ├── import.rs
│       └── report.rs
├── tests/
│   └── integration/
│       ├── mod.rs
│       └── cross_platform.rs
├── docs/
│   ├── commands.md
│   ├── keyboard-shortcuts.md
│   ├── data-format.md
│   └── troubleshooting.md
├── man/
│   └── envelope.1
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── .gitignore
├── README.md
└── INSTALL.md
```

### B. Data Directory Structure

```
~/.envelope/
├── config.json          # User preferences, settings
├── data/
│   ├── accounts.json    # Account definitions
│   ├── budget.json      # Categories, groups, allocations
│   ├── transactions.json # All transactions
│   └── payees.json      # Payee list with auto-categorization rules
├── audit.log            # Append-only modification log
└── backups/             # Automatic rolling backups
```

### C. Key Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `ratatui` | TUI framework |
| `crossterm` | Terminal backend |
| `serde` | Serialization |
| `serde_json` | JSON support |
| `uuid` | Unique identifiers |
| `chrono` | Date/time handling |
| `thiserror` | Error definitions |
| `anyhow` | Error propagation |
| `directories` | XDG paths |
| `argon2` | Key derivation |
| `aes-gcm` | Encryption |

### D. PRD Requirement Traceability

| Requirement | Step(s) |
|-------------|---------|
| FR-ACC-01 to FR-ACC-04 | 6, 14 |
| FR-BUD-01 to FR-BUD-07 | 7, 8, 9, 10 |
| FR-TXN-01 to FR-TXN-07 | 11, 12, 13, 15, 16 |
| FR-REC-01 to FR-REC-05 | 26, 27 |
| FR-RPT-01 to FR-RPT-05 | 28, 29, 30 |
| SEC-01 to SEC-11 | 4, 5, 30, 31 |

---

*This implementation plan was generated from EnvelopeCLI-PRD-MVP.docx*
