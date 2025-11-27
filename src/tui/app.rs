//! Application state for the TUI
//!
//! The App struct holds all state needed for rendering and handling events.

use crate::config::paths::EnvelopePaths;
use crate::config::settings::Settings;
use crate::models::{AccountId, BudgetPeriod, CategoryId, TransactionId};
use crate::storage::Storage;

use super::dialogs::adjustment::AdjustmentDialogState;
use super::dialogs::bulk_categorize::BulkCategorizeState;
use super::dialogs::move_funds::MoveFundsState;
use super::dialogs::reconcile_start::ReconcileStartState;
use super::dialogs::transaction::TransactionFormState;
use super::dialogs::unlock_confirm::UnlockConfirmState;
use super::views::reconcile::ReconciliationState;

/// Which view is currently active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveView {
    #[default]
    Accounts,
    Register,
    Budget,
    Reports,
    Reconcile,
}

/// Which panel currently has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    #[default]
    Sidebar,
    Main,
}

/// Mode of input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
    Command,
}

/// Currently active dialog (if any)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveDialog {
    None,
    AddTransaction,
    EditTransaction(TransactionId),
    MoveFunds,
    CommandPalette,
    Help,
    Confirm(String),
    BulkCategorize,
    ReconcileStart,
    UnlockConfirm(UnlockConfirmState),
    Adjustment,
}

impl Default for ActiveDialog {
    fn default() -> Self {
        Self::None
    }
}

/// Main application state
pub struct App<'a> {
    /// The storage layer
    pub storage: &'a Storage,

    /// Application settings
    pub settings: &'a Settings,

    /// Paths configuration
    pub paths: &'a EnvelopePaths,

    /// Whether the app should quit
    pub should_quit: bool,

    /// Currently active view
    pub active_view: ActiveView,

    /// Which panel is focused
    pub focused_panel: FocusedPanel,

    /// Current input mode
    pub input_mode: InputMode,

    /// Currently active dialog
    pub active_dialog: ActiveDialog,

    /// Selected account (if any)
    pub selected_account: Option<AccountId>,

    /// Selected account index in the list
    pub selected_account_index: usize,

    /// Selected transaction (if any)
    pub selected_transaction: Option<TransactionId>,

    /// Selected transaction index in the register
    pub selected_transaction_index: usize,

    /// Selected category (for budget view)
    pub selected_category: Option<CategoryId>,

    /// Selected category index
    pub selected_category_index: usize,

    /// Current budget period being viewed
    pub current_period: BudgetPeriod,

    /// Show archived accounts
    pub show_archived: bool,

    /// Multi-selection mode (for bulk operations)
    pub multi_select_mode: bool,

    /// Selected transaction IDs for bulk operations
    pub selected_transactions: Vec<TransactionId>,

    /// Scroll offset for the main view
    pub scroll_offset: usize,

    /// Status message to display
    pub status_message: Option<String>,

    /// Command palette input
    pub command_input: String,

    /// Filtered commands for palette
    pub command_results: Vec<usize>,

    /// Selected command index in palette
    pub selected_command_index: usize,

    /// Transaction form state
    pub transaction_form: TransactionFormState,

    /// Move funds dialog state
    pub move_funds_state: MoveFundsState,

    /// Bulk categorize dialog state
    pub bulk_categorize_state: BulkCategorizeState,

    /// Reconciliation view state
    pub reconciliation_state: ReconciliationState,

    /// Reconcile start dialog state
    pub reconcile_start_state: ReconcileStartState,

    /// Adjustment dialog state
    pub adjustment_dialog_state: AdjustmentDialogState,
}

impl<'a> App<'a> {
    /// Create a new App instance
    pub fn new(storage: &'a Storage, settings: &'a Settings, paths: &'a EnvelopePaths) -> Self {
        Self {
            storage,
            settings,
            paths,
            should_quit: false,
            active_view: ActiveView::default(),
            focused_panel: FocusedPanel::default(),
            input_mode: InputMode::default(),
            active_dialog: ActiveDialog::default(),
            selected_account: None,
            selected_account_index: 0,
            selected_transaction: None,
            selected_transaction_index: 0,
            selected_category: None,
            selected_category_index: 0,
            current_period: BudgetPeriod::current_month(),
            show_archived: false,
            multi_select_mode: false,
            selected_transactions: Vec::new(),
            scroll_offset: 0,
            status_message: None,
            command_input: String::new(),
            command_results: Vec::new(),
            selected_command_index: 0,
            transaction_form: TransactionFormState::new(),
            move_funds_state: MoveFundsState::new(),
            bulk_categorize_state: BulkCategorizeState::new(),
            reconciliation_state: ReconciliationState::new(),
            reconcile_start_state: ReconcileStartState::new(),
            adjustment_dialog_state: AdjustmentDialogState::default(),
        }
    }

    /// Request to quit the application
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Set a status message
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// Clear the status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Switch to a different view
    pub fn switch_view(&mut self, view: ActiveView) {
        self.active_view = view;
        self.scroll_offset = 0;

        // Reset selection based on view
        match view {
            ActiveView::Accounts => {
                self.selected_account_index = 0;
            }
            ActiveView::Register => {
                self.selected_transaction_index = 0;
            }
            ActiveView::Budget => {
                self.selected_category_index = 0;
            }
            ActiveView::Reports => {}
            ActiveView::Reconcile => {
                // Initialize reconciliation state if account is selected
                if let Some(account_id) = self.selected_account {
                    self.reconciliation_state.init_for_account(account_id);
                }
            }
        }
    }

    /// Toggle focus between sidebar and main panel
    pub fn toggle_panel_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Sidebar => FocusedPanel::Main,
            FocusedPanel::Main => FocusedPanel::Sidebar,
        };
    }

    /// Open a dialog
    pub fn open_dialog(&mut self, dialog: ActiveDialog) {
        self.active_dialog = dialog.clone();
        match &dialog {
            ActiveDialog::CommandPalette => {
                self.command_input.clear();
                self.input_mode = InputMode::Command;
            }
            ActiveDialog::AddTransaction => {
                // Reset form for new transaction
                self.transaction_form = TransactionFormState::new();
                self.transaction_form.set_focus(super::dialogs::transaction::TransactionField::Date);
                self.input_mode = InputMode::Editing;
            }
            ActiveDialog::EditTransaction(txn_id) => {
                // Load transaction data into form
                if let Ok(Some(txn)) = self.storage.transactions.get(*txn_id) {
                    let categories: Vec<_> = self.storage.categories
                        .get_all_categories()
                        .unwrap_or_default()
                        .iter()
                        .map(|c| (c.id, c.name.clone()))
                        .collect();
                    self.transaction_form = TransactionFormState::from_transaction(&txn, &categories);
                    self.transaction_form.set_focus(super::dialogs::transaction::TransactionField::Date);
                }
                self.input_mode = InputMode::Editing;
            }
            _ => {}
        }
    }

    /// Close the current dialog
    pub fn close_dialog(&mut self) {
        self.active_dialog = ActiveDialog::None;
        self.input_mode = InputMode::Normal;
    }

    /// Check if a dialog is active
    pub fn has_dialog(&self) -> bool {
        !matches!(self.active_dialog, ActiveDialog::None)
    }

    /// Move selection up in the current view
    pub fn move_up(&mut self) {
        match self.focused_panel {
            FocusedPanel::Sidebar => {
                if self.selected_account_index > 0 {
                    self.selected_account_index -= 1;
                }
            }
            FocusedPanel::Main => match self.active_view {
                ActiveView::Register => {
                    if self.selected_transaction_index > 0 {
                        self.selected_transaction_index -= 1;
                    }
                }
                ActiveView::Budget => {
                    if self.selected_category_index > 0 {
                        self.selected_category_index -= 1;
                    }
                }
                _ => {}
            },
        }
    }

    /// Move selection down in the current view
    pub fn move_down(&mut self, max: usize) {
        match self.focused_panel {
            FocusedPanel::Sidebar => {
                if self.selected_account_index < max.saturating_sub(1) {
                    self.selected_account_index += 1;
                }
            }
            FocusedPanel::Main => match self.active_view {
                ActiveView::Register => {
                    if self.selected_transaction_index < max.saturating_sub(1) {
                        self.selected_transaction_index += 1;
                    }
                }
                ActiveView::Budget => {
                    if self.selected_category_index < max.saturating_sub(1) {
                        self.selected_category_index += 1;
                    }
                }
                _ => {}
            },
        }
    }

    /// Go to previous budget period
    pub fn prev_period(&mut self) {
        self.current_period = self.current_period.prev();
    }

    /// Go to next budget period
    pub fn next_period(&mut self) {
        self.current_period = self.current_period.next();
    }

    /// Toggle multi-select mode
    pub fn toggle_multi_select(&mut self) {
        self.multi_select_mode = !self.multi_select_mode;
        if !self.multi_select_mode {
            self.selected_transactions.clear();
        }
    }

    /// Toggle selection of current transaction in multi-select mode
    pub fn toggle_transaction_selection(&mut self) {
        if let Some(txn_id) = self.selected_transaction {
            if self.selected_transactions.contains(&txn_id) {
                self.selected_transactions.retain(|&id| id != txn_id);
            } else {
                self.selected_transactions.push(txn_id);
            }
        }
    }
}
