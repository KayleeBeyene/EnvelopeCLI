//! Transaction entry/edit dialog
//!
//! Modal dialog for adding or editing transactions with form fields,
//! tab navigation, validation, and save/cancel functionality.

use chrono::{Local, NaiveDate};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::models::{CategoryId, Money, Transaction, TransactionStatus};
use crate::services::CategoryService;
use crate::tui::app::{ActiveDialog, App};
use crate::tui::layout::centered_rect;
use crate::tui::widgets::input::TextInput;

/// Which field is currently focused in the transaction form
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransactionField {
    #[default]
    Date,
    Payee,
    Category,
    Outflow,
    Inflow,
    Memo,
}

impl TransactionField {
    /// Get the next field (for Tab navigation)
    pub fn next(self) -> Self {
        match self {
            Self::Date => Self::Payee,
            Self::Payee => Self::Category,
            Self::Category => Self::Outflow,
            Self::Outflow => Self::Inflow,
            Self::Inflow => Self::Memo,
            Self::Memo => Self::Date,
        }
    }

    /// Get the previous field (for Shift+Tab navigation)
    pub fn prev(self) -> Self {
        match self {
            Self::Date => Self::Memo,
            Self::Payee => Self::Date,
            Self::Category => Self::Payee,
            Self::Outflow => Self::Category,
            Self::Inflow => Self::Outflow,
            Self::Memo => Self::Inflow,
        }
    }
}

/// State for the transaction form dialog
#[derive(Debug, Clone)]
pub struct TransactionFormState {
    /// Currently focused field
    pub focused_field: TransactionField,

    /// Date input
    pub date_input: TextInput,

    /// Payee input
    pub payee_input: TextInput,

    /// Category search input
    pub category_input: TextInput,

    /// Currently selected category ID
    pub selected_category: Option<CategoryId>,

    /// Category selection index (for dropdown)
    pub category_list_index: usize,

    /// Show category dropdown
    pub show_category_dropdown: bool,

    /// Outflow input (money going out - expenses)
    pub outflow_input: TextInput,

    /// Inflow input (money coming in - income)
    pub inflow_input: TextInput,

    /// Memo input
    pub memo_input: TextInput,

    /// Whether this is an edit (vs new transaction)
    pub is_edit: bool,

    /// Error message to display
    pub error_message: Option<String>,
}

impl Default for TransactionFormState {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionFormState {
    /// Create a new form state with default values
    pub fn new() -> Self {
        let today = Local::now().date_naive();
        Self {
            focused_field: TransactionField::Date,
            date_input: TextInput::new()
                .label("Date")
                .placeholder("YYYY-MM-DD")
                .content(today.format("%Y-%m-%d").to_string()),
            payee_input: TextInput::new()
                .label("Payee")
                .placeholder("Enter payee name"),
            category_input: TextInput::new()
                .label("Category")
                .placeholder("Type to search..."),
            selected_category: None,
            category_list_index: 0,
            show_category_dropdown: false,
            outflow_input: TextInput::new()
                .label("Outflow")
                .placeholder("(expense)"),
            inflow_input: TextInput::new()
                .label("Inflow")
                .placeholder("(income)"),
            memo_input: TextInput::new().label("Memo").placeholder("Optional note"),
            is_edit: false,
            error_message: None,
        }
    }

    /// Create form state pre-populated from an existing transaction
    pub fn from_transaction(txn: &Transaction, categories: &[(CategoryId, String)]) -> Self {
        let mut state = Self::new();
        state.is_edit = true;
        state.date_input = TextInput::new()
            .label("Date")
            .content(txn.date.format("%Y-%m-%d").to_string());
        state.payee_input = TextInput::new().label("Payee").content(&txn.payee_name);

        // Populate outflow/inflow based on amount sign
        let cents = txn.amount.cents();
        if cents < 0 {
            // Negative = outflow (expense)
            state.outflow_input = TextInput::new()
                .label("Outflow")
                .content(format!("{:.2}", (-cents) as f64 / 100.0));
            state.inflow_input = TextInput::new()
                .label("Inflow")
                .placeholder("0.00");
        } else if cents > 0 {
            // Positive = inflow (income)
            state.outflow_input = TextInput::new()
                .label("Outflow")
                .placeholder("0.00");
            state.inflow_input = TextInput::new()
                .label("Inflow")
                .content(format!("{:.2}", cents as f64 / 100.0));
        }

        state.memo_input = TextInput::new().label("Memo").content(&txn.memo);

        // Set category
        if let Some(cat_id) = txn.category_id {
            state.selected_category = Some(cat_id);
            if let Some((_, name)) = categories.iter().find(|(id, _)| *id == cat_id) {
                state.category_input = TextInput::new().label("Category").content(name);
            }
        }

        state
    }

    /// Move to the next field
    pub fn next_field(&mut self) {
        self.show_category_dropdown = false;
        self.focused_field = self.focused_field.next();
        self.update_focus();
    }

    /// Move to the previous field
    pub fn prev_field(&mut self) {
        self.show_category_dropdown = false;
        self.focused_field = self.focused_field.prev();
        self.update_focus();
    }

    /// Update which input has focus
    fn update_focus(&mut self) {
        self.date_input.focused = self.focused_field == TransactionField::Date;
        self.payee_input.focused = self.focused_field == TransactionField::Payee;
        self.category_input.focused = self.focused_field == TransactionField::Category;
        self.outflow_input.focused = self.focused_field == TransactionField::Outflow;
        self.inflow_input.focused = self.focused_field == TransactionField::Inflow;
        self.memo_input.focused = self.focused_field == TransactionField::Memo;

        // Show dropdown when category is focused
        if self.focused_field == TransactionField::Category {
            self.show_category_dropdown = true;
        }
    }

    /// Set focus to a specific field
    pub fn set_focus(&mut self, field: TransactionField) {
        self.focused_field = field;
        self.update_focus();
    }

    /// Get the currently focused input
    pub fn focused_input(&mut self) -> &mut TextInput {
        match self.focused_field {
            TransactionField::Date => &mut self.date_input,
            TransactionField::Payee => &mut self.payee_input,
            TransactionField::Category => &mut self.category_input,
            TransactionField::Outflow => &mut self.outflow_input,
            TransactionField::Inflow => &mut self.inflow_input,
            TransactionField::Memo => &mut self.memo_input,
        }
    }

    /// Validate the form and return any error
    pub fn validate(&self) -> Result<(), String> {
        // Validate date
        if NaiveDate::parse_from_str(self.date_input.value(), "%Y-%m-%d").is_err() {
            return Err("Invalid date format. Use YYYY-MM-DD".to_string());
        }

        // Validate outflow/inflow - at least one must have a value
        let outflow_str = self.outflow_input.value().trim();
        let inflow_str = self.inflow_input.value().trim();

        let has_outflow = !outflow_str.is_empty();
        let has_inflow = !inflow_str.is_empty();

        if !has_outflow && !has_inflow {
            return Err("Enter an outflow or inflow amount".to_string());
        }

        if has_outflow && has_inflow {
            return Err("Enter either outflow OR inflow, not both".to_string());
        }

        if has_outflow && Money::parse(outflow_str).is_err() {
            return Err("Invalid outflow format".to_string());
        }

        if has_inflow && Money::parse(inflow_str).is_err() {
            return Err("Invalid inflow format".to_string());
        }

        Ok(())
    }

    /// Build a transaction from the form state
    pub fn build_transaction(
        &self,
        account_id: crate::models::AccountId,
    ) -> Result<Transaction, String> {
        self.validate()?;

        let date = NaiveDate::parse_from_str(self.date_input.value(), "%Y-%m-%d")
            .map_err(|_| "Invalid date")?;

        // Calculate amount from outflow/inflow
        let outflow_str = self.outflow_input.value().trim();
        let inflow_str = self.inflow_input.value().trim();

        let amount = if !outflow_str.is_empty() {
            // Outflow = negative amount (expense)
            let parsed = Money::parse(outflow_str).map_err(|_| "Invalid outflow")?;
            -parsed
        } else {
            // Inflow = positive amount (income)
            Money::parse(inflow_str).map_err(|_| "Invalid inflow")?
        };

        let mut txn = Transaction::with_details(
            account_id,
            date,
            amount,
            self.payee_input.value(),
            self.selected_category,
            self.memo_input.value(),
        );

        txn.status = TransactionStatus::Pending;

        Ok(txn)
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

/// Render the transaction dialog
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(70, 70, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let title = match &app.active_dialog {
        ActiveDialog::AddTransaction => " Add Transaction ",
        ActiveDialog::EditTransaction(_) => " Edit Transaction ",
        _ => " Transaction ",
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

    // Layout: fields + category dropdown + buttons
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Date
            Constraint::Length(1), // Payee
            Constraint::Length(1), // Category input
            Constraint::Length(6), // Category dropdown
            Constraint::Length(1), // Outflow
            Constraint::Length(1), // Inflow
            Constraint::Length(1), // Memo
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Error
            Constraint::Length(1), // Buttons
            Constraint::Min(0),    // Remaining
        ])
        .split(inner);

    // Extract values we need from form (to avoid borrow conflicts)
    let date_value = app.transaction_form.date_input.value().to_string();
    let date_focused = app.transaction_form.focused_field == TransactionField::Date;
    let date_cursor = app.transaction_form.date_input.cursor;
    let date_placeholder = app.transaction_form.date_input.placeholder.clone();

    let payee_value = app.transaction_form.payee_input.value().to_string();
    let payee_focused = app.transaction_form.focused_field == TransactionField::Payee;
    let payee_cursor = app.transaction_form.payee_input.cursor;
    let payee_placeholder = app.transaction_form.payee_input.placeholder.clone();

    let outflow_value = app.transaction_form.outflow_input.value().to_string();
    let outflow_focused = app.transaction_form.focused_field == TransactionField::Outflow;
    let outflow_cursor = app.transaction_form.outflow_input.cursor;
    let outflow_placeholder = app.transaction_form.outflow_input.placeholder.clone();

    let inflow_value = app.transaction_form.inflow_input.value().to_string();
    let inflow_focused = app.transaction_form.focused_field == TransactionField::Inflow;
    let inflow_cursor = app.transaction_form.inflow_input.cursor;
    let inflow_placeholder = app.transaction_form.inflow_input.placeholder.clone();

    let memo_value = app.transaction_form.memo_input.value().to_string();
    let memo_focused = app.transaction_form.focused_field == TransactionField::Memo;
    let memo_cursor = app.transaction_form.memo_input.cursor;
    let memo_placeholder = app.transaction_form.memo_input.placeholder.clone();

    let error_message = app.transaction_form.error_message.clone();

    // Render date field
    render_field_simple(
        frame,
        chunks[0],
        "Date",
        &date_value,
        date_focused,
        date_cursor,
        &date_placeholder,
    );

    // Render payee field
    render_field_simple(
        frame,
        chunks[1],
        "Payee",
        &payee_value,
        payee_focused,
        payee_cursor,
        &payee_placeholder,
    );

    // Render category field (needs app for category lookup)
    render_category_field(frame, app, chunks[2], chunks[3]);

    // Render outflow field
    render_field_simple(
        frame,
        chunks[4],
        "Outflow",
        &outflow_value,
        outflow_focused,
        outflow_cursor,
        &outflow_placeholder,
    );

    // Render inflow field
    render_field_simple(
        frame,
        chunks[5],
        "Inflow",
        &inflow_value,
        inflow_focused,
        inflow_cursor,
        &inflow_placeholder,
    );

    // Render memo field
    render_field_simple(
        frame,
        chunks[6],
        "Memo",
        &memo_value,
        memo_focused,
        memo_cursor,
        &memo_placeholder,
    );

    // Render error message if any
    if let Some(ref error) = error_message {
        let error_line = Line::from(Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        ));
        frame.render_widget(Paragraph::new(error_line), chunks[8]);
    }

    // Render buttons/hints
    let hints = Line::from(vec![
        Span::styled("[Tab]", Style::default().fg(Color::Yellow)),
        Span::raw(" Next  "),
        Span::styled("[Shift+Tab]", Style::default().fg(Color::Yellow)),
        Span::raw(" Prev  "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Save  "),
        Span::styled("[Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Cancel"),
    ]);
    frame.render_widget(Paragraph::new(hints), chunks[9]);
}

/// Render a single form field with extracted values
fn render_field_simple(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    focused: bool,
    cursor: usize,
    placeholder: &str,
) {
    // Label
    let label_style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let label_span = Span::styled(format!("{:>10}: ", label), label_style);

    // Value with cursor if focused
    let value_style = if focused {
        Style::default().fg(Color::White)
    } else if value.is_empty() {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let display_value = if value.is_empty() && !focused {
        placeholder.to_string()
    } else {
        value.to_string()
    };

    let mut spans = vec![label_span];

    if focused {
        // Show value with cursor
        let cursor_pos = cursor.min(display_value.len());
        let (before, after) = display_value.split_at(cursor_pos);

        spans.push(Span::styled(before.to_string(), value_style));

        // Cursor character
        let cursor_char = after.chars().next().unwrap_or(' ');
        spans.push(Span::styled(
            cursor_char.to_string(),
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ));

        // Rest after cursor
        if after.len() > 1 {
            spans.push(Span::styled(after[1..].to_string(), value_style));
        }
    } else {
        spans.push(Span::styled(display_value, value_style));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Render the category field with dropdown
fn render_category_field(frame: &mut Frame, app: &mut App, input_area: Rect, dropdown_area: Rect) {
    let form = &app.transaction_form;
    let focused = form.focused_field == TransactionField::Category;

    // Render the input line
    let label_style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    // Show selected category name or search input
    let display_value = if let Some(cat_id) = form.selected_category {
        // Try to get category name
        if let Ok(categories) = app.storage.categories.get_all_categories() {
            categories
                .iter()
                .find(|c| c.id == cat_id)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| form.category_input.value().to_string())
        } else {
            form.category_input.value().to_string()
        }
    } else if form.category_input.value().is_empty() && !focused {
        form.category_input.placeholder.clone()
    } else {
        form.category_input.value().to_string()
    };

    let value_style = if focused {
        Style::default().fg(Color::White)
    } else if display_value.is_empty() || display_value == form.category_input.placeholder {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let mut spans = vec![Span::styled(format!("{:>10}: ", "Category"), label_style)];

    if focused && form.selected_category.is_none() {
        // Show input with cursor
        let cursor_pos = form.category_input.cursor.min(display_value.len());
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
        if focused && form.selected_category.is_some() {
            spans.push(Span::styled(
                " (Backspace to clear)",
                Style::default().fg(Color::Yellow),
            ));
        }
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), input_area);

    // Render dropdown if focused and no category selected
    if focused {
        render_category_dropdown(frame, app, dropdown_area);
    }
}

/// Render the category dropdown list
fn render_category_dropdown(frame: &mut Frame, app: &mut App, area: Rect) {
    let category_service = CategoryService::new(app.storage);
    let categories = category_service.list_categories().unwrap_or_default();

    // Filter categories based on search input
    let search = app.transaction_form.category_input.value().to_lowercase();
    let filtered: Vec<_> = categories
        .iter()
        .filter(|c| search.is_empty() || c.name.to_lowercase().contains(&search))
        .take(5)
        .collect();

    if filtered.is_empty() {
        let hint = if search.is_empty() {
            "No categories available"
        } else {
            "No matching categories"
        };
        let text = Paragraph::new(hint).style(Style::default().fg(Color::Yellow));
        frame.render_widget(text, area);
        return;
    }

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|cat| {
            ListItem::new(Line::from(Span::styled(
                format!("  {}", cat.name),
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
    let idx = app
        .transaction_form
        .category_list_index
        .min(filtered.len().saturating_sub(1));
    state.select(Some(idx));

    frame.render_stateful_widget(list, area, &mut state);
}

/// Handle key input for the transaction dialog
/// Returns true if the key was handled, false otherwise
pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::{KeyCode, KeyModifiers};

    let form = &mut app.transaction_form;

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
            // If in category dropdown and category is focused, select the category
            if form.focused_field == TransactionField::Category && form.selected_category.is_none()
            {
                select_category_from_dropdown(app);
                return true;
            }

            // Otherwise, try to save
            if let Err(e) = save_transaction(app) {
                app.transaction_form.set_error(e);
            }
            return true;
        }

        KeyCode::Up => {
            if form.focused_field == TransactionField::Category && form.selected_category.is_none()
            {
                if form.category_list_index > 0 {
                    form.category_list_index -= 1;
                }
                return true;
            }
        }

        KeyCode::Down => {
            if form.focused_field == TransactionField::Category && form.selected_category.is_none()
            {
                form.category_list_index += 1;
                return true;
            }
        }

        KeyCode::Backspace => {
            form.clear_error();

            // If category is selected and we're in category field, clear it
            if form.focused_field == TransactionField::Category && form.selected_category.is_some()
            {
                form.selected_category = None;
                form.category_input.clear();
                return true;
            }

            // Normal backspace on focused input
            form.focused_input().backspace();
            return true;
        }

        KeyCode::Delete => {
            form.clear_error();
            form.focused_input().delete();
            return true;
        }

        KeyCode::Left => {
            form.focused_input().move_left();
            return true;
        }

        KeyCode::Right => {
            form.focused_input().move_right();
            return true;
        }

        KeyCode::Home => {
            form.focused_input().move_start();
            return true;
        }

        KeyCode::End => {
            form.focused_input().move_end();
            return true;
        }

        KeyCode::Char(c) => {
            form.clear_error();

            // If category is selected and we're typing, clear it first
            if form.focused_field == TransactionField::Category && form.selected_category.is_some()
            {
                form.selected_category = None;
                form.category_input.clear();
            }

            form.focused_input().insert(c);

            // Reset category list index when typing in category field
            if form.focused_field == TransactionField::Category {
                form.category_list_index = 0;
            }

            return true;
        }

        _ => {}
    }

    false
}

/// Select the currently highlighted category from the dropdown
fn select_category_from_dropdown(app: &mut App) {
    let category_service = CategoryService::new(app.storage);
    let categories = category_service.list_categories().unwrap_or_default();

    let search = app.transaction_form.category_input.value().to_lowercase();
    let filtered: Vec<_> = categories
        .iter()
        .filter(|c| search.is_empty() || c.name.to_lowercase().contains(&search))
        .take(5)
        .collect();

    let idx = app
        .transaction_form
        .category_list_index
        .min(filtered.len().saturating_sub(1));
    if let Some(cat) = filtered.get(idx) {
        app.transaction_form.selected_category = Some(cat.id);
        app.transaction_form.category_input = TextInput::new().label("Category").content(&cat.name);
        app.transaction_form.next_field(); // Move to next field after selection
    }
}

/// Save the transaction
fn save_transaction(app: &mut App) -> Result<(), String> {
    // Validate form
    app.transaction_form.validate()?;

    // Get account ID
    let account_id = app.selected_account.ok_or("No account selected")?;

    // Build transaction
    let txn = app.transaction_form.build_transaction(account_id)?;

    // Check if edit or new
    let is_edit = matches!(app.active_dialog, ActiveDialog::EditTransaction(_));

    if is_edit {
        if let ActiveDialog::EditTransaction(txn_id) = app.active_dialog {
            // Update existing transaction
            if let Ok(Some(mut existing)) = app.storage.transactions.get(txn_id) {
                existing.date = txn.date;
                existing.amount = txn.amount;
                existing.payee_name = txn.payee_name;
                existing.category_id = txn.category_id;
                existing.memo = txn.memo;
                existing.updated_at = chrono::Utc::now();

                app.storage
                    .transactions
                    .upsert(existing)
                    .map_err(|e| e.to_string())?;
            }
        }
    } else {
        // Create new transaction
        app.storage
            .transactions
            .upsert(txn)
            .map_err(|e| e.to_string())?;
    }

    // Save to disk
    app.storage.transactions.save().map_err(|e| e.to_string())?;

    // Close dialog
    app.close_dialog();
    app.set_status(if is_edit {
        "Transaction updated"
    } else {
        "Transaction created"
    });

    Ok(())
}
