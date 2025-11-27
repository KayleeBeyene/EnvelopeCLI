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
    // Get account count for bounds checking
    let account_count = app.storage.accounts.get_active()
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

        _ => {}
    }

    Ok(())
}

/// Handle keys when main panel is focused
fn handle_main_panel_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match app.active_view {
        ActiveView::Accounts => handle_accounts_view_key(app, key),
        ActiveView::Register => handle_register_view_key(app, key),
        ActiveView::Budget => handle_budget_view_key(app, key),
        ActiveView::Reports => handle_reports_view_key(app, key),
    }
}

/// Handle keys in the accounts view
fn handle_accounts_view_key(app: &mut App, key: KeyEvent) -> Result<()> {
    let account_count = app.storage.accounts.get_active()
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
        _ => {}
    }

    Ok(())
}

/// Handle keys in the register view
fn handle_register_view_key(app: &mut App, key: KeyEvent) -> Result<()> {
    // Get transaction count for the selected account
    let txn_count = if let Some(account_id) = app.selected_account {
        app.storage.transactions.get_by_account(account_id)
            .map(|t| t.len())
            .unwrap_or(0)
    } else {
        0
    };

    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down(txn_count);
            // Update selected transaction
            if let Some(account_id) = app.selected_account {
                if let Ok(txns) = app.storage.transactions.get_by_account(account_id) {
                    if let Some(txn) = txns.get(app.selected_transaction_index) {
                        app.selected_transaction = Some(txn.id);
                    }
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
            if let Some(account_id) = app.selected_account {
                if let Ok(txns) = app.storage.transactions.get_by_account(account_id) {
                    if let Some(txn) = txns.get(app.selected_transaction_index) {
                        app.selected_transaction = Some(txn.id);
                    }
                }
            }
        }

        // Page navigation
        KeyCode::Char('G') => {
            // Go to bottom
            if txn_count > 0 {
                app.selected_transaction_index = txn_count - 1;
            }
        }
        KeyCode::Char('g') => {
            // Go to top (gg in vim, but we'll use single g)
            app.selected_transaction_index = 0;
        }

        // Add transaction
        KeyCode::Char('a') | KeyCode::Char('n') => {
            app.open_dialog(ActiveDialog::AddTransaction);
        }

        // Edit transaction
        KeyCode::Char('e') | KeyCode::Enter => {
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
                app.open_dialog(ActiveDialog::Confirm("Delete this transaction?".to_string()));
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

/// Handle keys in the budget view
fn handle_budget_view_key(app: &mut App, key: KeyEvent) -> Result<()> {
    // Get category count
    let category_count = app.storage.categories.get_all_categories()
        .map(|c| c.len())
        .unwrap_or(0);

    match key.code {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down(category_count);
            if let Ok(categories) = app.storage.categories.get_all_categories() {
                if let Some(cat) = categories.get(app.selected_category_index) {
                    app.selected_category = Some(cat.id);
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
            if let Ok(categories) = app.storage.categories.get_all_categories() {
                if let Some(cat) = categories.get(app.selected_category_index) {
                    app.selected_category = Some(cat.id);
                }
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

        // Quick budget (edit inline) - would need editing mode
        KeyCode::Enter => {
            // Could implement inline editing here
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
        ActiveDialog::None => {}
    }
    Ok(())
}
