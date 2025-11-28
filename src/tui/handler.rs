//! Event handler for the TUI
//!
//! Routes keyboard and mouse events to the appropriate handlers
//! based on the current application state.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{ActiveDialog, ActiveView, App, FocusedPanel, InputMode};
use super::event::Event;

/// Handle an incoming event
pub fn handle_event(app: &mut App, event: Event) -> Result<()> {
    match event {
        Event::Key(key) => handle_key_event(app, key),
        Event::Mouse(_mouse) => {
            // Mouse handling can be added later
            Ok(())
        }
        Event::Tick => Ok(()),
        Event::Resize(_, _) => Ok(()),
    }
}

/// Handle a key event
fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    // DEBUG: Log every key at the top level
    app.set_status(format!(
        "KEY: {:?} | mode={:?} | panel={:?} | view={:?} | dialog={:?}",
        key.code,
        app.input_mode,
        app.focused_panel,
        app.active_view,
        app.has_dialog()
    ));

    // Check if we're in a dialog first
    if app.has_dialog() {
        return handle_dialog_key(app, key);
    }

    // Check input mode
    match app.input_mode {
        InputMode::Normal => handle_normal_key(app, key),
        InputMode::Editing => handle_editing_key(app, key),
        InputMode::Command => handle_command_key(app, key),
    }
}

/// Handle keys in normal mode
fn handle_normal_key(app: &mut App, key: KeyEvent) -> Result<()> {
    // Global keys (work everywhere)
    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.quit();
            return Ok(());
        }

        // Help
        KeyCode::Char('?') => {
            app.open_dialog(ActiveDialog::Help);
            return Ok(());
        }

        // Command palette
        KeyCode::Char(':') | KeyCode::Char('/') => {
            app.open_dialog(ActiveDialog::CommandPalette);
            return Ok(());
        }

        // Panel navigation
        KeyCode::Tab => {
            app.toggle_panel_focus();
            return Ok(());
        }
        KeyCode::Char('h') | KeyCode::Left if key.modifiers.is_empty() => {
            if app.focused_panel == FocusedPanel::Main {
                app.focused_panel = FocusedPanel::Sidebar;
                return Ok(());
            }
        }
        KeyCode::Char('l') | KeyCode::Right if key.modifiers.is_empty() => {
            if app.focused_panel == FocusedPanel::Sidebar {
                app.focused_panel = FocusedPanel::Main;
                app.ensure_selection_initialized();
                return Ok(());
            }
        }

        _ => {}
    }

    // View-specific keys
    match app.focused_panel {
        FocusedPanel::Sidebar => handle_sidebar_key(app, key),
        FocusedPanel::Main => handle_main_panel_key(app, key),
    }
}

/// Handle keys when sidebar is focused
fn handle_sidebar_key(app: &mut App, key: KeyEvent) -> Result<()> {
    // DEBUG: Show we're in sidebar
    app.set_status(format!("DEBUG: sidebar key={:?}", key.code));

    // Get account count for bounds checking
    let account_count = app
        .storage
        .accounts
        .get_active()
        .map(|a| a.len())
        .unwrap_or(0);

    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down(account_count);
            // Update selected account
            if let Ok(accounts) = app.storage.accounts.get_active() {
                if let Some(account) = accounts.get(app.selected_account_index) {
                    app.selected_account = Some(account.id);
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
            // Update selected account
            if let Ok(accounts) = app.storage.accounts.get_active() {
                if let Some(account) = accounts.get(app.selected_account_index) {
                    app.selected_account = Some(account.id);
                }
            }
        }

        // Select account and view register
        KeyCode::Enter => {
            if let Ok(accounts) = app.storage.accounts.get_active() {
                if let Some(account) = accounts.get(app.selected_account_index) {
                    app.selected_account = Some(account.id);
                    app.switch_view(ActiveView::Register);
                    app.focused_panel = FocusedPanel::Main;
                }
            }
        }

        // View switching from sidebar
        KeyCode::Char('1') => app.switch_view(ActiveView::Accounts),
        KeyCode::Char('2') => app.switch_view(ActiveView::Budget),
        KeyCode::Char('3') => app.switch_view(ActiveView::Reports),

        // Toggle archived accounts
        KeyCode::Char('A') => {
            app.show_archived = !app.show_archived;
        }

        // Add new account
        KeyCode::Char('a') | KeyCode::Char('n') => {
            app.open_dialog(ActiveDialog::AddAccount);
        }

        // Edit selected account
        KeyCode::Char('e') => {
            if let Ok(accounts) = app.storage.accounts.get_active() {
                if let Some(account) = accounts.get(app.selected_account_index) {
                    app.open_dialog(ActiveDialog::EditAccount(account.id));
                }
            }
        }

        _ => {}
    }

    Ok(())
}

/// Handle keys when main panel is focused
fn handle_main_panel_key(app: &mut App, key: KeyEvent) -> Result<()> {
    // DEBUG: Show which view we're in
    app.set_status(format!(
        "DEBUG: main_panel key={:?}, view={:?}",
        key.code, app.active_view
    ));

    match app.active_view {
        ActiveView::Accounts => handle_accounts_view_key(app, key),
        ActiveView::Register => handle_register_view_key(app, key),
        ActiveView::Budget => handle_budget_view_key(app, key),
        ActiveView::Reports => handle_reports_view_key(app, key),
        ActiveView::Reconcile => handle_reconcile_view_key(app, key),
    }
}

/// Handle keys in the accounts view
fn handle_accounts_view_key(app: &mut App, key: KeyEvent) -> Result<()> {
    let account_count = app
        .storage
        .accounts
        .get_active()
        .map(|a| a.len())
        .unwrap_or(0);

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down(account_count);
            if let Ok(accounts) = app.storage.accounts.get_active() {
                if let Some(account) = accounts.get(app.selected_account_index) {
                    app.selected_account = Some(account.id);
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
            if let Ok(accounts) = app.storage.accounts.get_active() {
                if let Some(account) = accounts.get(app.selected_account_index) {
                    app.selected_account = Some(account.id);
                }
            }
        }
        KeyCode::Enter => {
            // Switch to register view for selected account
            app.switch_view(ActiveView::Register);
        }
        // Add new account
        KeyCode::Char('a') | KeyCode::Char('n') => {
            app.open_dialog(ActiveDialog::AddAccount);
        }
        // Edit selected account
        KeyCode::Char('e') => {
            if let Ok(accounts) = app.storage.accounts.get_active() {
                if let Some(account) = accounts.get(app.selected_account_index) {
                    app.open_dialog(ActiveDialog::EditAccount(account.id));
                }
            }
        }
        _ => {}
    }

    Ok(())
}

/// Get sorted transactions for an account (matches display order)
fn get_sorted_transactions(app: &App) -> Vec<crate::models::Transaction> {
    if let Some(account_id) = app.selected_account {
        let mut txns = app
            .storage
            .transactions
            .get_by_account(account_id)
            .unwrap_or_default();
        // Sort by date descending (matches render order)
        txns.sort_by(|a, b| b.date.cmp(&a.date));
        txns
    } else {
        Vec::new()
    }
}

/// Handle keys in the register view
fn handle_register_view_key(app: &mut App, key: KeyEvent) -> Result<()> {
    // Get sorted transactions (matches display order)
    let txns = get_sorted_transactions(app);
    let txn_count = txns.len();

    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down(txn_count);
            // Update selected transaction from sorted list
            if let Some(txn) = txns.get(app.selected_transaction_index) {
                app.selected_transaction = Some(txn.id);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
            // Update selected transaction from sorted list
            if let Some(txn) = txns.get(app.selected_transaction_index) {
                app.selected_transaction = Some(txn.id);
            }
        }

        // Page navigation
        KeyCode::Char('G') => {
            // Go to bottom
            if txn_count > 0 {
                app.selected_transaction_index = txn_count - 1;
                if let Some(txn) = txns.get(app.selected_transaction_index) {
                    app.selected_transaction = Some(txn.id);
                }
            }
        }
        KeyCode::Char('g') => {
            // Go to top (gg in vim, but we'll use single g)
            app.selected_transaction_index = 0;
            if let Some(txn) = txns.first() {
                app.selected_transaction = Some(txn.id);
            }
        }

        // Add transaction
        KeyCode::Char('a') | KeyCode::Char('n') => {
            app.open_dialog(ActiveDialog::AddTransaction);
        }

        // Edit transaction
        KeyCode::Char('e') => {
            // DEBUG: Force initialize selection and try edit
            if app.selected_transaction.is_none() {
                let txns = get_sorted_transactions(app);
                if let Some(txn) = txns.get(app.selected_transaction_index) {
                    app.selected_transaction = Some(txn.id);
                }
            }
            if let Some(txn_id) = app.selected_transaction {
                app.open_dialog(ActiveDialog::EditTransaction(txn_id));
            }
        }
        KeyCode::Enter => {
            if app.selected_transaction.is_none() {
                let txns = get_sorted_transactions(app);
                if let Some(txn) = txns.get(app.selected_transaction_index) {
                    app.selected_transaction = Some(txn.id);
                }
            }
            if let Some(txn_id) = app.selected_transaction {
                app.open_dialog(ActiveDialog::EditTransaction(txn_id));
            }
        }

        // Clear transaction (toggle)
        KeyCode::Char('c') => {
            if let Some(txn_id) = app.selected_transaction {
                // Toggle cleared status
                if let Ok(Some(txn)) = app.storage.transactions.get(txn_id) {
                    use crate::models::TransactionStatus;
                    let new_status = match txn.status {
                        TransactionStatus::Pending => TransactionStatus::Cleared,
                        TransactionStatus::Cleared => TransactionStatus::Pending,
                        TransactionStatus::Reconciled => TransactionStatus::Reconciled,
                    };
                    if new_status != txn.status {
                        let mut txn = txn.clone();
                        txn.set_status(new_status);
                        let _ = app.storage.transactions.upsert(txn);
                        let _ = app.storage.transactions.save();
                        app.set_status(format!("Transaction marked as {}", new_status));
                    }
                }
            }
        }

        // Delete transaction
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.selected_transaction.is_some() {
                app.open_dialog(ActiveDialog::Confirm(
                    "Delete this transaction?".to_string(),
                ));
            }
        }

        // Multi-select mode
        KeyCode::Char('v') => {
            app.toggle_multi_select();
            if app.multi_select_mode {
                app.set_status("Multi-select mode ON");
            } else {
                app.set_status("Multi-select mode OFF");
            }
        }

        // Toggle selection in multi-select mode
        KeyCode::Char(' ') if app.multi_select_mode => {
            app.toggle_transaction_selection();
        }

        // Bulk categorize
        KeyCode::Char('C') if app.multi_select_mode && !app.selected_transactions.is_empty() => {
            app.open_dialog(ActiveDialog::BulkCategorize);
        }

        _ => {}
    }

    Ok(())
}

/// Get categories in visual order (grouped by group, same as render)
fn get_categories_in_visual_order(app: &App) -> Vec<crate::models::Category> {
    let groups = app.storage.categories.get_all_groups().unwrap_or_default();
    let all_categories = app
        .storage
        .categories
        .get_all_categories()
        .unwrap_or_default();

    let mut result = Vec::new();
    for group in &groups {
        let group_cats: Vec<_> = all_categories
            .iter()
            .filter(|c| c.group_id == group.id)
            .cloned()
            .collect();
        result.extend(group_cats);
    }
    result
}

/// Handle keys in the budget view
fn handle_budget_view_key(app: &mut App, key: KeyEvent) -> Result<()> {
    // Get categories in visual order (matches display)
    let categories = get_categories_in_visual_order(app);
    let category_count = categories.len();

    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down(category_count);
            if let Some(cat) = categories.get(app.selected_category_index) {
                app.selected_category = Some(cat.id);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
            if let Some(cat) = categories.get(app.selected_category_index) {
                app.selected_category = Some(cat.id);
            }
        }

        // Period navigation
        KeyCode::Char('[') | KeyCode::Char('H') => {
            app.prev_period();
        }
        KeyCode::Char(']') | KeyCode::Char('L') => {
            app.next_period();
        }

        // Move funds
        KeyCode::Char('m') => {
            app.open_dialog(ActiveDialog::MoveFunds);
        }

        // Edit budget for selected category
        KeyCode::Enter => {
            // Initialize and open the edit budget dialog
            if let Some(cat) = categories.get(app.selected_category_index) {
                let budget_service = crate::services::BudgetService::new(app.storage);
                let summary = budget_service
                    .get_category_summary(cat.id, &app.current_period)
                    .unwrap_or_else(|_| crate::models::CategoryBudgetSummary::empty(cat.id));

                app.edit_budget_state
                    .init(cat.id, cat.name.clone(), summary.budgeted);
                app.open_dialog(ActiveDialog::EditBudget);
            }
        }

        _ => {}
    }

    Ok(())
}

/// Handle keys in the reports view
fn handle_reports_view_key(_app: &mut App, _key: KeyEvent) -> Result<()> {
    // Reports view keys will be added later
    Ok(())
}

/// Handle keys in the reconcile view
fn handle_reconcile_view_key(app: &mut App, key: KeyEvent) -> Result<()> {
    // Delegate to the reconcile view's key handler
    super::views::reconcile::handle_key(app, key.code);
    Ok(())
}

/// Handle keys in editing mode
fn handle_editing_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        _ => {
            // Pass to dialog if active
        }
    }
    Ok(())
}

/// Handle keys in command mode (command palette)
fn handle_command_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.close_dialog();
        }
        KeyCode::Enter => {
            // Execute selected command
            app.close_dialog();
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
            // Filter commands based on input
        }
        KeyCode::Backspace => {
            app.command_input.pop();
        }
        KeyCode::Up => {
            if app.selected_command_index > 0 {
                app.selected_command_index -= 1;
            }
        }
        KeyCode::Down => {
            app.selected_command_index += 1;
        }
        _ => {}
    }
    Ok(())
}

/// Handle keys when a dialog is open
fn handle_dialog_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match &app.active_dialog {
        ActiveDialog::Help => {
            // Close help on any key
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Enter => {
                    app.close_dialog();
                }
                _ => {}
            }
        }
        ActiveDialog::CommandPalette => {
            handle_command_key(app, key)?;
        }
        ActiveDialog::Confirm(_) => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    // Execute confirmed action
                    app.close_dialog();
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    app.close_dialog();
                }
                _ => {}
            }
        }
        ActiveDialog::AddTransaction | ActiveDialog::EditTransaction(_) => {
            // Delegate to transaction dialog key handler
            super::dialogs::transaction::handle_key(app, key);
        }
        ActiveDialog::MoveFunds => {
            super::dialogs::move_funds::handle_key(app, key);
        }
        ActiveDialog::BulkCategorize => {
            super::dialogs::bulk_categorize::handle_key(app, key);
        }
        ActiveDialog::ReconcileStart => {
            match key.code {
                KeyCode::Esc => {
                    app.close_dialog();
                }
                KeyCode::Enter => {
                    // Start reconciliation
                    app.close_dialog();
                    app.switch_view(ActiveView::Reconcile);
                }
                _ => {
                    super::dialogs::reconcile_start::handle_key(app, key.code);
                }
            }
        }
        ActiveDialog::UnlockConfirm(_) => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    // Unlock the transaction
                    app.close_dialog();
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    app.close_dialog();
                }
                _ => {}
            }
        }
        ActiveDialog::Adjustment => {
            match key.code {
                KeyCode::Esc => {
                    app.close_dialog();
                }
                KeyCode::Enter => {
                    // Create adjustment
                    app.close_dialog();
                }
                _ => {
                    super::dialogs::adjustment::handle_key(app, key.code);
                }
            }
        }
        ActiveDialog::EditBudget => {
            super::dialogs::edit_budget::handle_key(app, key);
        }
        ActiveDialog::AddAccount | ActiveDialog::EditAccount(_) => {
            super::dialogs::account::handle_key(app, key);
        }
        ActiveDialog::None => {}
    }
    Ok(())
}
