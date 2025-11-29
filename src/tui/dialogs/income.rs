//! Income dialog
//!
//! A dialog for setting expected income for a budget period.
//! Allows users to set, view, and remove income expectations.

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::models::{BudgetPeriod, Money};
use crate::services::IncomeService;
use crate::tui::app::App;
use crate::tui::layout::centered_rect_fixed;

/// Which field is focused in the income dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IncomeField {
    #[default]
    Amount,
    Notes,
}

impl IncomeField {
    pub fn next(self) -> Self {
        match self {
            Self::Amount => Self::Notes,
            Self::Notes => Self::Amount,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Amount => Self::Notes,
            Self::Notes => Self::Amount,
        }
    }
}

/// State for the income dialog
#[derive(Debug, Clone, Default)]
pub struct IncomeFormState {
    /// The period being edited
    pub period: Option<BudgetPeriod>,
    /// Which field is focused
    pub focused_field: IncomeField,
    /// Amount input
    pub amount_input: String,
    /// Amount cursor position
    pub amount_cursor: usize,
    /// Notes input
    pub notes_input: String,
    /// Notes cursor position
    pub notes_cursor: usize,
    /// Whether there's an existing income expectation
    pub has_existing: bool,
    /// Current expected income (for display)
    pub current_amount: Option<Money>,
    /// Error message
    pub error_message: Option<String>,
}

impl IncomeFormState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the dialog for a period
    pub fn init_for_period(&mut self, period: &BudgetPeriod, storage: &crate::storage::Storage) {
        self.period = Some(period.clone());
        self.focused_field = IncomeField::Amount;
        self.error_message = None;

        let service = IncomeService::new(storage);
        if let Some(expectation) = service.get_income_expectation(period) {
            self.has_existing = true;
            self.current_amount = Some(expectation.expected_amount);
            let cents = expectation.expected_amount.cents();
            if cents == 0 {
                self.amount_input = String::new();
            } else {
                self.amount_input = format!("{:.2}", cents as f64 / 100.0);
            }
            self.amount_cursor = self.amount_input.len();
            self.notes_input = expectation.notes.clone();
            self.notes_cursor = self.notes_input.len();
        } else {
            self.has_existing = false;
            self.current_amount = None;
            self.amount_input = String::new();
            self.amount_cursor = 0;
            self.notes_input = String::new();
            self.notes_cursor = 0;
        }
    }

    /// Reset the state
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Set focus to a field
    pub fn set_focus(&mut self, field: IncomeField) {
        self.focused_field = field;
    }

    /// Move to next field
    pub fn next_field(&mut self) {
        self.focused_field = self.focused_field.next();
    }

    /// Move to previous field
    pub fn prev_field(&mut self) {
        self.focused_field = self.focused_field.prev();
    }

    /// Insert character into current field
    pub fn insert_char(&mut self, c: char) {
        match self.focused_field {
            IncomeField::Amount => {
                if c.is_ascii_digit() || c == '.' {
                    self.amount_input.insert(self.amount_cursor, c);
                    self.amount_cursor += 1;
                    self.error_message = None;
                }
            }
            IncomeField::Notes => {
                self.notes_input.insert(self.notes_cursor, c);
                self.notes_cursor += 1;
                self.error_message = None;
            }
        }
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        match self.focused_field {
            IncomeField::Amount => {
                if self.amount_cursor > 0 {
                    self.amount_cursor -= 1;
                    self.amount_input.remove(self.amount_cursor);
                    self.error_message = None;
                }
            }
            IncomeField::Notes => {
                if self.notes_cursor > 0 {
                    self.notes_cursor -= 1;
                    self.notes_input.remove(self.notes_cursor);
                    self.error_message = None;
                }
            }
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        match self.focused_field {
            IncomeField::Amount => {
                if self.amount_cursor > 0 {
                    self.amount_cursor -= 1;
                }
            }
            IncomeField::Notes => {
                if self.notes_cursor > 0 {
                    self.notes_cursor -= 1;
                }
            }
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        match self.focused_field {
            IncomeField::Amount => {
                if self.amount_cursor < self.amount_input.len() {
                    self.amount_cursor += 1;
                }
            }
            IncomeField::Notes => {
                if self.notes_cursor < self.notes_input.len() {
                    self.notes_cursor += 1;
                }
            }
        }
    }

    /// Clear current field
    pub fn clear_field(&mut self) {
        match self.focused_field {
            IncomeField::Amount => {
                self.amount_input.clear();
                self.amount_cursor = 0;
            }
            IncomeField::Notes => {
                self.notes_input.clear();
                self.notes_cursor = 0;
            }
        }
        self.error_message = None;
    }

    /// Parse the amount input
    pub fn parse_amount(&self) -> Result<Money, String> {
        if self.amount_input.trim().is_empty() {
            return Err("Amount is required".to_string());
        }
        Money::parse(&self.amount_input).map_err(|_| "Invalid amount format".to_string())
    }

    /// Get notes (None if empty)
    pub fn get_notes(&self) -> Option<String> {
        let trimmed = self.notes_input.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    /// Set error message
    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
    }
}

/// Render the income dialog
pub fn render(frame: &mut Frame, app: &App) {
    let state = &app.income_form;

    let height = if state.has_existing { 14 } else { 12 };
    let area = centered_rect_fixed(55, height, frame.area());
    frame.render_widget(Clear, area);

    let title = format!(" Expected Income: {} ", app.current_period);
    let block = Block::default()
        .title(title)
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut constraints = vec![
        Constraint::Length(1), // Current income (if exists)
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Amount label
        Constraint::Length(1), // Amount input
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Notes label
        Constraint::Length(1), // Notes input
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Error
        Constraint::Length(1), // Instructions
        Constraint::Min(0),
    ];

    if !state.has_existing {
        constraints.remove(0); // Remove current income row if none exists
        constraints.remove(0); // Remove spacer after current income
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let mut row = 0;

    // Current income (if exists)
    if state.has_existing {
        if let Some(current) = state.current_amount {
            let current_line = Line::from(vec![
                Span::styled("Current:   ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}", current),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
            frame.render_widget(Paragraph::new(current_line), chunks[row]);
        }
        row += 2; // Skip current and spacer
    }

    // Amount label
    let amount_label_style = if state.focused_field == IncomeField::Amount {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };
    frame.render_widget(
        Paragraph::new(Span::styled("Amount:", amount_label_style)),
        chunks[row],
    );
    row += 1;

    // Amount input with cursor
    let amount_line = render_input_with_cursor(
        "$",
        &state.amount_input,
        state.amount_cursor,
        state.focused_field == IncomeField::Amount,
    );
    frame.render_widget(Paragraph::new(amount_line), chunks[row]);
    row += 2; // Skip input and spacer

    // Notes label
    let notes_label_style = if state.focused_field == IncomeField::Notes {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };
    frame.render_widget(
        Paragraph::new(Span::styled("Notes (optional):", notes_label_style)),
        chunks[row],
    );
    row += 1;

    // Notes input with cursor
    let notes_line = render_input_with_cursor(
        "",
        &state.notes_input,
        state.notes_cursor,
        state.focused_field == IncomeField::Notes,
    );
    frame.render_widget(Paragraph::new(notes_line), chunks[row]);
    row += 2; // Skip input and spacer

    // Error message
    if let Some(ref error) = state.error_message {
        let error_line = Line::from(Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        ));
        frame.render_widget(Paragraph::new(error_line), chunks[row]);
    }
    row += 1;

    // Instructions
    let mut instructions = vec![
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Save  "),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::raw(" Cancel  "),
        Span::styled("[Tab]", Style::default().fg(Color::Cyan)),
        Span::raw(" Fields"),
    ];

    if state.has_existing {
        instructions.push(Span::raw("  "));
        instructions.push(Span::styled("[Del]", Style::default().fg(Color::Magenta)));
        instructions.push(Span::raw(" Remove"));
    }

    frame.render_widget(Paragraph::new(Line::from(instructions)), chunks[row]);
}

fn render_input_with_cursor(
    prefix: &str,
    value: &str,
    cursor: usize,
    focused: bool,
) -> Line<'static> {
    let mut spans = vec![];

    if !prefix.is_empty() {
        spans.push(Span::raw(prefix.to_string()));
    }

    if focused {
        let cursor_pos = cursor.min(value.len());
        let (before, after) = value.split_at(cursor_pos);

        spans.push(Span::styled(
            before.to_string(),
            Style::default().fg(Color::White),
        ));

        let cursor_char = after.chars().next().unwrap_or(' ');
        spans.push(Span::styled(
            cursor_char.to_string(),
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ));

        if after.len() > 1 {
            spans.push(Span::styled(
                after[1..].to_string(),
                Style::default().fg(Color::White),
            ));
        }
    } else {
        spans.push(Span::styled(
            value.to_string(),
            Style::default().fg(Color::White),
        ));
    }

    Line::from(spans)
}

/// Handle key events for the income dialog
pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::{KeyCode, KeyModifiers};

    match key.code {
        KeyCode::Esc => {
            app.income_form.reset();
            app.close_dialog();
            true
        }

        KeyCode::Tab => {
            app.income_form.next_field();
            true
        }

        KeyCode::BackTab => {
            app.income_form.prev_field();
            true
        }

        KeyCode::Enter => {
            if let Err(e) = save_income(app) {
                app.income_form.set_error(e);
            }
            true
        }

        KeyCode::Delete => {
            if app.income_form.has_existing {
                if let Err(e) = remove_income(app) {
                    app.income_form.set_error(e);
                }
            }
            true
        }

        KeyCode::Down | KeyCode::Char('j') if key.modifiers.is_empty() => {
            app.income_form.next_field();
            true
        }

        KeyCode::Up | KeyCode::Char('k')
            if key.modifiers.is_empty() && app.income_form.focused_field == IncomeField::Notes =>
        {
            app.income_form.prev_field();
            true
        }

        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.income_form.clear_field();
            true
        }

        KeyCode::Char(c) => {
            app.income_form.insert_char(c);
            true
        }

        KeyCode::Backspace => {
            app.income_form.backspace();
            true
        }

        KeyCode::Left => {
            app.income_form.move_left();
            true
        }

        KeyCode::Right => {
            app.income_form.move_right();
            true
        }

        _ => false,
    }
}

fn save_income(app: &mut App) -> Result<(), String> {
    let period = app.income_form.period.clone().ok_or("No period selected")?;
    let amount = app.income_form.parse_amount()?;
    let notes = app.income_form.get_notes();

    let service = IncomeService::new(app.storage);
    service
        .set_expected_income(&period, amount, notes)
        .map_err(|e| e.to_string())?;

    app.income_form.reset();
    app.close_dialog();
    app.set_status(format!("Expected income for {} set to {}", period, amount));

    Ok(())
}

fn remove_income(app: &mut App) -> Result<(), String> {
    let period = app.income_form.period.clone().ok_or("No period selected")?;

    let service = IncomeService::new(app.storage);
    if service
        .delete_expected_income(&period)
        .map_err(|e| e.to_string())?
    {
        app.income_form.reset();
        app.close_dialog();
        app.set_status(format!("Expected income removed for {}", period));
        Ok(())
    } else {
        Err("No income expectation to remove".to_string())
    }
}
