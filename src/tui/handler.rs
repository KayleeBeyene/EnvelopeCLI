//! Event handler for the TUI
//!
//! Routes keyboard and mouse events to the appropriate handlers
//! based on the current application state.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{ActiveDialog, ActiveView, App, FocusedPanel, InputMode};
use super::commands::{CommandAction, COMMANDS};
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

        // Add new category
        KeyCode::Char('a') => {
            app.open_dialog(ActiveDialog::AddCategory);
        }

        // Add new category group
        KeyCode::Char('A') => {
            app.open_dialog(ActiveDialog::AddGroup);
        }

        // Open unified budget dialog (period budget + target)
        KeyCode::Enter | KeyCode::Char('b') | KeyCode::Char('t') => {
            if let Some(cat) = categories.get(app.selected_category_index) {
                app.selected_category = Some(cat.id);
                app.open_dialog(ActiveDialog::Budget);
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
            // Get filtered commands (same logic as render)
            let filtered_commands: Vec<&crate::tui::commands::Command> = COMMANDS
                .iter()
                .filter(|cmd| {
                    if app.command_input.is_empty() {
                        true
                    } else {
                        let query = app.command_input.to_lowercase();
                        cmd.name.to_lowercase().contains(&query)
                            || cmd.description.to_lowercase().contains(&query)
                    }
                })
                .collect();

            // Get the selected command
            if !filtered_commands.is_empty() {
                let selected_idx = app
                    .selected_command_index
                    .min(filtered_commands.len().saturating_sub(1));
                let command = filtered_commands[selected_idx];
                let action = command.action;

                // Close dialog first
                app.close_dialog();

                // Execute the command action
                execute_command_action(app, action)?;
            } else {
                app.close_dialog();
            }
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
            // Reset selection when input changes
            app.selected_command_index = 0;
        }
        KeyCode::Backspace => {
            app.command_input.pop();
            // Reset selection when input changes
            app.selected_command_index = 0;
        }
        KeyCode::Up => {
            if app.selected_command_index > 0 {
                app.selected_command_index -= 1;
            }
        }
        KeyCode::Down => {
            // Get filtered count to bound selection
            let filtered_count = COMMANDS
                .iter()
                .filter(|cmd| {
                    if app.command_input.is_empty() {
                        true
                    } else {
                        let query = app.command_input.to_lowercase();
                        cmd.name.to_lowercase().contains(&query)
                            || cmd.description.to_lowercase().contains(&query)
                    }
                })
                .count();
            if app.selected_command_index + 1 < filtered_count {
                app.selected_command_index += 1;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Execute a command action from the command palette
fn execute_command_action(app: &mut App, action: CommandAction) -> Result<()> {
    match action {
        // Navigation
        CommandAction::ViewAccounts => {
            app.switch_view(ActiveView::Accounts);
        }
        CommandAction::ViewBudget => {
            app.switch_view(ActiveView::Budget);
        }
        CommandAction::ViewReports => {
            app.switch_view(ActiveView::Reports);
        }
        CommandAction::ViewRegister => {
            app.switch_view(ActiveView::Register);
        }

        // Account operations
        CommandAction::AddAccount => {
            app.open_dialog(ActiveDialog::AddAccount);
        }
        CommandAction::EditAccount => {
            if let Ok(accounts) = app.storage.accounts.get_active() {
                if let Some(account) = accounts.get(app.selected_account_index) {
                    app.open_dialog(ActiveDialog::EditAccount(account.id));
                }
            }
        }
        CommandAction::ArchiveAccount => {
            // Archive selected account with confirmation
            if let Ok(accounts) = app.storage.accounts.get_active() {
                if let Some(account) = accounts.get(app.selected_account_index) {
                    app.open_dialog(ActiveDialog::Confirm(format!(
                        "Archive account '{}'?",
                        account.name
                    )));
                } else {
                    app.set_status("No account selected".to_string());
                }
            }
        }

        // Transaction operations
        CommandAction::AddTransaction => {
            app.open_dialog(ActiveDialog::AddTransaction);
        }
        CommandAction::EditTransaction => {
            if let Some(tx_id) = app.selected_transaction {
                app.open_dialog(ActiveDialog::EditTransaction(tx_id));
            } else {
                app.set_status("No transaction selected".to_string());
            }
        }
        CommandAction::DeleteTransaction => {
            if app.selected_transaction.is_some() {
                app.open_dialog(ActiveDialog::Confirm("Delete transaction?".to_string()));
            } else {
                app.set_status("No transaction selected".to_string());
            }
        }
        CommandAction::ClearTransaction => {
            // Toggle cleared status for selected transaction
            if let Some(txn_id) = app.selected_transaction {
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
            } else {
                app.set_status("No transaction selected".to_string());
            }
        }

        // Budget operations
        CommandAction::MoveFunds => {
            app.open_dialog(ActiveDialog::MoveFunds);
        }
        CommandAction::AssignBudget => {
            // Open unified budget dialog for the selected category
            if app.selected_category.is_some() {
                app.open_dialog(ActiveDialog::Budget);
            } else {
                app.set_status("No category selected. Switch to Budget view first.".to_string());
            }
        }
        CommandAction::NextPeriod => {
            app.next_period();
        }
        CommandAction::PrevPeriod => {
            app.prev_period();
        }

        // Category operations
        CommandAction::AddCategory => {
            // Initialize category form with available groups
            let groups: Vec<_> = app
                .storage
                .categories
                .get_all_groups()
                .unwrap_or_default()
                .into_iter()
                .map(|g| (g.id, g.name.clone()))
                .collect();
            app.category_form.init_with_groups(groups);
            app.open_dialog(ActiveDialog::AddCategory);
        }
        CommandAction::AddGroup => {
            // Reset group form
            app.group_form = super::dialogs::group::GroupFormState::new();
            app.open_dialog(ActiveDialog::AddGroup);
        }
        CommandAction::EditCategory => {
            // Open EditCategory dialog for the selected category
            if let Some(category_id) = app.selected_category {
                app.open_dialog(ActiveDialog::EditCategory(category_id));
            } else {
                app.set_status("No category selected. Switch to Budget view first.".to_string());
            }
        }
        CommandAction::DeleteCategory => {
            // Delete selected category with confirmation
            if let Some(category_id) = app.selected_category {
                if let Ok(Some(category)) = app.storage.categories.get_category(category_id) {
                    app.open_dialog(ActiveDialog::Confirm(format!(
                        "Delete category '{}'?",
                        category.name
                    )));
                }
            } else {
                app.set_status("No category selected".to_string());
            }
        }

        // General
        CommandAction::Help => {
            app.open_dialog(ActiveDialog::Help);
        }
        CommandAction::Quit => {
            app.quit();
        }
        CommandAction::Refresh => {
            // Reload all data from disk
            if let Err(e) = app.storage.accounts.load() {
                app.set_status(format!("Failed to refresh accounts: {}", e));
                return Ok(());
            }
            if let Err(e) = app.storage.transactions.load() {
                app.set_status(format!("Failed to refresh transactions: {}", e));
                return Ok(());
            }
            if let Err(e) = app.storage.categories.load() {
                app.set_status(format!("Failed to refresh categories: {}", e));
                return Ok(());
            }
            if let Err(e) = app.storage.budget.load() {
                app.set_status(format!("Failed to refresh budget: {}", e));
                return Ok(());
            }
            app.set_status("Data refreshed from disk".to_string());
        }
        CommandAction::ToggleArchived => {
            app.show_archived = !app.show_archived;
        }

        // Target operations
        CommandAction::AutoFillTargets => {
            use crate::services::BudgetService;
            let budget_service = BudgetService::new(app.storage);
            match budget_service.auto_fill_all_targets(&app.current_period) {
                Ok(allocations) => {
                    if allocations.is_empty() {
                        app.set_status("No targets to auto-fill".to_string());
                    } else {
                        let count = allocations.len();
                        let plural = if count == 1 { "category" } else { "categories" };
                        app.set_status(format!("{} {} updated from targets", count, plural));
                    }
                }
                Err(e) => {
                    app.set_status(format!("Auto-fill failed: {}", e));
                }
            }
        }
    }
    Ok(())
}

/// Handle keys when a dialog is open
fn handle_dialog_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match &app.active_dialog {
        ActiveDialog::Help => {
            // Close help on any key
            app.close_dialog();
        }
        ActiveDialog::CommandPalette => {
            handle_command_key(app, key)?;
        }
        ActiveDialog::Confirm(msg) => {
            let msg = msg.clone();
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    // Execute confirmed action based on the message
                    app.close_dialog();
                    execute_confirmed_action(app, &msg)?;
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
        ActiveDialog::Budget => {
            super::dialogs::budget::handle_key(app, key);
        }
        ActiveDialog::AddAccount | ActiveDialog::EditAccount(_) => {
            super::dialogs::account::handle_key(app, key);
        }
        ActiveDialog::AddCategory | ActiveDialog::EditCategory(_) => {
            super::dialogs::category::handle_key(app, key);
        }
        ActiveDialog::AddGroup => {
            super::dialogs::group::handle_key(app, key);
        }
        ActiveDialog::None => {}
    }
    Ok(())
}

/// Execute an action after user confirmation
fn execute_confirmed_action(app: &mut App, message: &str) -> Result<()> {
    // Delete transaction
    if message.contains("Delete") && message.contains("transaction") {
        if let Some(txn_id) = app.selected_transaction {
            if let Err(e) = app.storage.transactions.delete(txn_id) {
                app.set_status(format!("Failed to delete: {}", e));
            } else {
                let _ = app.storage.transactions.save();
                app.selected_transaction = None;
                app.set_status("Transaction deleted".to_string());
            }
        }
    }
    // Archive account
    else if message.contains("Archive account") {
        if let Ok(accounts) = app.storage.accounts.get_active() {
            if let Some(account) = accounts.get(app.selected_account_index) {
                let mut account = account.clone();
                account.archive();
                if let Err(e) = app.storage.accounts.upsert(account.clone()) {
                    app.set_status(format!("Failed to archive: {}", e));
                } else {
                    let _ = app.storage.accounts.save();
                    app.set_status(format!("Account '{}' archived", account.name));
                    // Reset selection
                    app.selected_account_index = 0;
                    if let Ok(active) = app.storage.accounts.get_active() {
                        app.selected_account = active.first().map(|a| a.id);
                    }
                }
            }
        }
    }
    // Delete category
    else if message.contains("Delete category") {
        if let Some(category_id) = app.selected_category {
            use crate::services::CategoryService;
            let category_service = CategoryService::new(app.storage);
            match category_service.delete_category(category_id) {
                Ok(()) => {
                    app.set_status("Category deleted".to_string());
                    app.selected_category = None;
                    app.selected_category_index = 0;
                }
                Err(e) => {
                    app.set_status(format!("Failed to delete: {}", e));
                }
            }
        }
    }

    Ok(())
}
