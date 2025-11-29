//! Account entry dialog
//!
//! Modal dialog for adding new accounts with form fields,
//! tab navigation, validation, and save/cancel functionality.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::models::{Account, AccountType, Money};
use crate::tui::app::App;
use crate::tui::layout::centered_rect;
use crate::tui::widgets::input::TextInput;

/// Account types available for selection
const ACCOUNT_TYPES: &[AccountType] = &[
    AccountType::Checking,
    AccountType::Savings,
    AccountType::Credit,
    AccountType::Cash,
    AccountType::Investment,
    AccountType::LineOfCredit,
    AccountType::Other,
];

/// Which field is currently focused in the account form
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccountField {
    #[default]
    Name,
    AccountType,
    StartingBalance,
    OnBudget,
}

impl AccountField {
    /// Get the next field (for Tab navigation)
    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::AccountType,
            Self::AccountType => Self::StartingBalance,
            Self::StartingBalance => Self::OnBudget,
            Self::OnBudget => Self::Name,
        }
    }

    /// Get the previous field (for Shift+Tab navigation)
    pub fn prev(self) -> Self {
        match self {
            Self::Name => Self::OnBudget,
            Self::AccountType => Self::Name,
            Self::StartingBalance => Self::AccountType,
            Self::OnBudget => Self::StartingBalance,
        }
    }
}

/// State for the account form dialog
#[derive(Debug, Clone)]
pub struct AccountFormState {
    /// Currently focused field
    pub focused_field: AccountField,

    /// Name input
    pub name_input: TextInput,

    /// Selected account type index
    pub account_type_index: usize,

    /// Starting balance input
    pub balance_input: TextInput,

    /// Whether account is on budget
    pub on_budget: bool,

    /// Whether this is an edit (vs new account)
    pub is_edit: bool,

    /// Account ID being edited (if editing)
    pub editing_account_id: Option<crate::models::AccountId>,

    /// Error message to display
    pub error_message: Option<String>,
}

impl Default for AccountFormState {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountFormState {
    /// Create a new form state with default values
    pub fn new() -> Self {
        Self {
            focused_field: AccountField::Name,
            name_input: TextInput::new().label("Name").placeholder("Account name"),
            account_type_index: 0, // Checking
            balance_input: TextInput::new()
                .label("Balance")
                .placeholder("0.00")
                .content("0.00"),
            on_budget: true,
            is_edit: false,
            editing_account_id: None,
            error_message: None,
        }
    }

    /// Create form state pre-populated from an existing account
    pub fn from_account(account: &Account) -> Self {
        // Find the account type index
        let account_type_index = ACCOUNT_TYPES
            .iter()
            .position(|t| *t == account.account_type)
            .unwrap_or(0);

        Self {
            focused_field: AccountField::Name,
            name_input: TextInput::new().label("Name").content(&account.name),
            account_type_index,
            balance_input: TextInput::new().label("Balance").content(format!(
                "{:.2}",
                account.starting_balance.cents() as f64 / 100.0
            )),
            on_budget: account.on_budget,
            is_edit: true,
            editing_account_id: Some(account.id),
            error_message: None,
        }
    }

    /// Move to the next field
    pub fn next_field(&mut self) {
        self.focused_field = self.focused_field.next();
    }

    /// Move to the previous field
    pub fn prev_field(&mut self) {
        self.focused_field = self.focused_field.prev();
    }

    /// Set focus to a specific field
    pub fn set_focus(&mut self, field: AccountField) {
        self.focused_field = field;
    }

    /// Get the currently focused text input (if applicable)
    pub fn focused_input(&mut self) -> Option<&mut TextInput> {
        match self.focused_field {
            AccountField::Name => Some(&mut self.name_input),
            AccountField::StartingBalance => Some(&mut self.balance_input),
            _ => None,
        }
    }

    /// Get selected account type
    pub fn selected_account_type(&self) -> AccountType {
        ACCOUNT_TYPES
            .get(self.account_type_index)
            .copied()
            .unwrap_or(AccountType::Checking)
    }

    /// Move to next account type
    pub fn next_account_type(&mut self) {
        self.account_type_index = (self.account_type_index + 1) % ACCOUNT_TYPES.len();
    }

    /// Move to previous account type
    pub fn prev_account_type(&mut self) {
        if self.account_type_index == 0 {
            self.account_type_index = ACCOUNT_TYPES.len() - 1;
        } else {
            self.account_type_index -= 1;
        }
    }

    /// Toggle on-budget status
    pub fn toggle_on_budget(&mut self) {
        self.on_budget = !self.on_budget;
    }

    /// Validate the form and return any error
    pub fn validate(&self) -> Result<(), String> {
        let name = self.name_input.value().trim();
        if name.is_empty() {
            return Err("Account name is required".to_string());
        }
        if name.len() > 100 {
            return Err("Account name too long (max 100 chars)".to_string());
        }

        // Validate balance format
        let balance_str = self.balance_input.value().trim();
        if !balance_str.is_empty() && Money::parse(balance_str).is_err() {
            return Err("Invalid balance format".to_string());
        }

        Ok(())
    }

    /// Build an account from the form state
    pub fn build_account(&self) -> Result<Account, String> {
        self.validate()?;

        let name = self.name_input.value().trim().to_string();
        let account_type = self.selected_account_type();

        let balance_str = self.balance_input.value().trim();
        let mut starting_balance = if balance_str.is_empty() {
            Money::zero()
        } else {
            Money::parse(balance_str).map_err(|_| "Invalid balance")?
        };

        // For liability accounts (credit cards, lines of credit), balances represent
        // debt owed and should be stored as negative values. Users naturally enter
        // positive numbers when specifying debt, so we negate them.
        if account_type.is_liability() && starting_balance.cents() > 0 {
            starting_balance = Money::from_cents(-starting_balance.cents());
        }

        let mut account = Account::with_starting_balance(name, account_type, starting_balance);
        account.on_budget = self.on_budget;

        Ok(account)
    }

    /// Clear any error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Set an error message
    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
    }
}

/// Render the account dialog
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 50, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let title = if app.account_form.is_edit {
        " Edit Account "
    } else {
        " Add Account "
    };

    let block = Block::default()
        .title(title)
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(block, area);

    // Inner area for content
    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    // Layout: fields + buttons
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Name
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Account Type label
            Constraint::Length(5), // Account Type list
            Constraint::Length(1), // Starting Balance
            Constraint::Length(1), // On Budget
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Error
            Constraint::Length(1), // Buttons
            Constraint::Min(0),    // Remaining
        ])
        .split(inner);

    // Extract values to avoid borrow conflicts
    let name_value = app.account_form.name_input.value().to_string();
    let name_focused = app.account_form.focused_field == AccountField::Name;
    let name_cursor = app.account_form.name_input.cursor;
    let name_placeholder = app.account_form.name_input.placeholder.clone();

    let balance_value = app.account_form.balance_input.value().to_string();
    let balance_focused = app.account_form.focused_field == AccountField::StartingBalance;
    let balance_cursor = app.account_form.balance_input.cursor;
    let balance_placeholder = app.account_form.balance_input.placeholder.clone();

    let type_focused = app.account_form.focused_field == AccountField::AccountType;
    let budget_focused = app.account_form.focused_field == AccountField::OnBudget;
    let on_budget = app.account_form.on_budget;
    let error_message = app.account_form.error_message.clone();

    // Render name field
    render_text_field(
        frame,
        chunks[0],
        "Name",
        &name_value,
        name_focused,
        name_cursor,
        &name_placeholder,
    );

    // Render account type label
    let type_label_style = if type_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };
    let type_label = Paragraph::new(Line::from(vec![
        Span::styled("Type: ", type_label_style),
        Span::styled("(↑/↓ to change)", Style::default().fg(Color::White)),
    ]));
    frame.render_widget(type_label, chunks[2]);

    // Render account type list
    render_account_type_list(frame, app, chunks[3]);

    // Render starting balance field
    render_text_field(
        frame,
        chunks[4],
        "Balance",
        &balance_value,
        balance_focused,
        balance_cursor,
        &balance_placeholder,
    );

    // Render on budget toggle
    let budget_label_style = if budget_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };
    let budget_value = if on_budget { "[x] Yes" } else { "[ ] No" };
    let budget_hint = if budget_focused {
        " (Space to toggle)"
    } else {
        ""
    };
    let budget_line = Line::from(vec![
        Span::styled("On Budget: ", budget_label_style),
        Span::styled(budget_value, Style::default().fg(Color::White)),
        Span::styled(budget_hint, Style::default().fg(Color::White)),
    ]);
    frame.render_widget(Paragraph::new(budget_line), chunks[5]);

    // Render error message if any
    if let Some(ref error) = error_message {
        let error_line = Line::from(Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        ));
        frame.render_widget(Paragraph::new(error_line), chunks[7]);
    }

    // Render buttons/hints
    let hints = Line::from(vec![
        Span::styled("[Tab]", Style::default().fg(Color::White)),
        Span::raw(" Next  "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Save  "),
        Span::styled("[Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Cancel"),
    ]);
    frame.render_widget(Paragraph::new(hints), chunks[8]);
}

/// Render a text field
fn render_text_field(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    focused: bool,
    cursor: usize,
    placeholder: &str,
) {
    let label_style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let label_span = Span::styled(format!("{}: ", label), label_style);

    let value_style = Style::default().fg(Color::White);

    let display_value = if value.is_empty() && !focused {
        placeholder.to_string()
    } else {
        value.to_string()
    };

    let mut spans = vec![label_span];

    if focused {
        let cursor_pos = cursor.min(display_value.len());
        let (before, after) = display_value.split_at(cursor_pos);

        spans.push(Span::styled(before.to_string(), value_style));

        let cursor_char = after.chars().next().unwrap_or(' ');
        spans.push(Span::styled(
            cursor_char.to_string(),
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ));

        if after.len() > 1 {
            spans.push(Span::styled(after[1..].to_string(), value_style));
        }
    } else {
        spans.push(Span::styled(display_value, value_style));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Render the account type selection list
fn render_account_type_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let form = &app.account_form;
    let focused = form.focused_field == AccountField::AccountType;

    let items: Vec<ListItem> = ACCOUNT_TYPES
        .iter()
        .map(|t| {
            ListItem::new(Line::from(Span::styled(
                format!("  {}", t),
                Style::default().fg(Color::White),
            )))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(form.account_type_index));

    if focused {
        frame.render_stateful_widget(list, area, &mut state);
    } else {
        // Show hint when not focused
        let hint = Paragraph::new("  (Tab to this field to select)")
            .style(Style::default().fg(Color::White));
        frame.render_widget(hint, area);
    }
}

/// Handle key input for the account dialog
pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::{KeyCode, KeyModifiers};

    let form = &mut app.account_form;

    match key.code {
        KeyCode::Esc => {
            app.close_dialog();
            return true;
        }

        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                form.prev_field();
            } else {
                form.next_field();
            }
            return true;
        }

        KeyCode::BackTab => {
            form.prev_field();
            return true;
        }

        KeyCode::Enter => {
            // Try to save
            if let Err(e) = save_account(app) {
                app.account_form.set_error(e);
            }
            return true;
        }

        KeyCode::Up => {
            if form.focused_field == AccountField::AccountType {
                form.prev_account_type();
                return true;
            }
        }

        KeyCode::Down => {
            if form.focused_field == AccountField::AccountType {
                form.next_account_type();
                return true;
            }
        }

        KeyCode::Char(' ') => {
            if form.focused_field == AccountField::OnBudget {
                form.toggle_on_budget();
                return true;
            }
            // Otherwise fall through to character input
        }

        KeyCode::Backspace => {
            form.clear_error();
            if let Some(input) = form.focused_input() {
                input.backspace();
            }
            return true;
        }

        KeyCode::Delete => {
            form.clear_error();
            if let Some(input) = form.focused_input() {
                input.delete();
            }
            return true;
        }

        KeyCode::Left => {
            if let Some(input) = form.focused_input() {
                input.move_left();
            }
            return true;
        }

        KeyCode::Right => {
            if let Some(input) = form.focused_input() {
                input.move_right();
            }
            return true;
        }

        KeyCode::Home => {
            if let Some(input) = form.focused_input() {
                input.move_start();
            }
            return true;
        }

        KeyCode::End => {
            if let Some(input) = form.focused_input() {
                input.move_end();
            }
            return true;
        }

        KeyCode::Char(c) => {
            form.clear_error();
            if let Some(input) = form.focused_input() {
                input.insert(c);
            }
            return true;
        }

        _ => {}
    }

    false
}

/// Save the account
fn save_account(app: &mut App) -> Result<(), String> {
    // Validate form
    app.account_form.validate()?;

    let is_edit = app.account_form.is_edit;
    let editing_id = app.account_form.editing_account_id;

    if is_edit {
        // Update existing account
        if let Some(account_id) = editing_id {
            if let Ok(Some(mut existing)) = app.storage.accounts.get(account_id) {
                existing.name = app.account_form.name_input.value().trim().to_string();
                existing.account_type = app.account_form.selected_account_type();
                existing.on_budget = app.account_form.on_budget;

                // Update starting balance
                let balance_str = app.account_form.balance_input.value().trim();
                let mut new_balance = if balance_str.is_empty() {
                    Money::zero()
                } else {
                    Money::parse(balance_str).map_err(|_| "Invalid balance")?
                };

                // For liability accounts, negate positive balances (debt is stored as negative)
                if existing.account_type.is_liability() && new_balance.cents() > 0 {
                    new_balance = Money::from_cents(-new_balance.cents());
                }
                existing.starting_balance = new_balance;

                existing.updated_at = chrono::Utc::now();

                let account_name = existing.name.clone();
                app.storage
                    .accounts
                    .upsert(existing)
                    .map_err(|e| e.to_string())?;

                app.storage.accounts.save().map_err(|e| e.to_string())?;
                app.close_dialog();
                app.set_status(format!("Account '{}' updated", account_name));
            }
        }
    } else {
        // Build new account
        let account = app.account_form.build_account()?;
        let account_name = account.name.clone();

        // Save to storage
        app.storage
            .accounts
            .upsert(account)
            .map_err(|e| e.to_string())?;

        // Save to disk
        app.storage.accounts.save().map_err(|e| e.to_string())?;

        // Close dialog
        app.close_dialog();
        app.set_status(format!("Account '{}' created", account_name));
    }

    Ok(())
}
