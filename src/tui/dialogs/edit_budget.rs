//! Edit budget dialog
//!
//! Dialog to edit the budgeted amount for a category in a period

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::models::{CategoryId, Money};
use crate::services::BudgetService;
use crate::tui::app::App;
use crate::tui::layout::centered_rect_fixed;

/// State for the edit budget dialog
#[derive(Debug, Clone, Default)]
pub struct EditBudgetState {
    /// The category being edited
    pub category_id: Option<CategoryId>,
    /// Category name for display
    pub category_name: String,
    /// Current budgeted amount
    pub current_amount: Money,
    /// Input value (as string for editing)
    pub amount_input: String,
    /// Cursor position in the input
    pub cursor_pos: usize,
    /// Error message
    pub error_message: Option<String>,
}

impl EditBudgetState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the dialog for a category
    pub fn init(&mut self, category_id: CategoryId, category_name: String, current_amount: Money) {
        self.category_id = Some(category_id);
        self.category_name = category_name;
        self.current_amount = current_amount;
        // Pre-fill with current amount (without $ sign, formatted as decimal)
        let cents = current_amount.cents();
        if cents == 0 {
            self.amount_input = String::new();
        } else {
            self.amount_input = format!("{:.2}", cents as f64 / 100.0);
        }
        self.cursor_pos = self.amount_input.len();
        self.error_message = None;
    }

    /// Reset the state
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Insert character at cursor
    pub fn insert_char(&mut self, c: char) {
        if c.is_ascii_digit() || c == '.' {
            self.amount_input.insert(self.cursor_pos, c);
            self.cursor_pos += 1;
            self.error_message = None;
        }
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.amount_input.remove(self.cursor_pos);
            self.error_message = None;
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        if self.cursor_pos < self.amount_input.len() {
            self.cursor_pos += 1;
        }
    }

    /// Clear the input
    pub fn clear_input(&mut self) {
        self.amount_input.clear();
        self.cursor_pos = 0;
        self.error_message = None;
    }

    /// Parse the input amount
    pub fn parse_amount(&self) -> Result<Money, String> {
        if self.amount_input.trim().is_empty() {
            return Ok(Money::zero());
        }
        Money::parse(&self.amount_input).map_err(|_| "Invalid amount format".to_string())
    }

    /// Set error message
    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
    }
}

/// Render the edit budget dialog
pub fn render(frame: &mut Frame, app: &App) {
    let area = centered_rect_fixed(50, 12, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let state = &app.edit_budget_state;

    let block = Block::default()
        .title(" Edit Budget ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Category name
            Constraint::Length(1), // Period
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Current amount
            Constraint::Length(1), // New amount label
            Constraint::Length(1), // Amount input
            Constraint::Length(1), // Error
            Constraint::Length(1), // Instructions
            Constraint::Min(0),    // Remaining
        ])
        .split(inner);

    // Category name
    let category_line = Line::from(vec![
        Span::styled("Category: ", Style::default().fg(Color::Yellow)),
        Span::styled(&state.category_name, Style::default().fg(Color::White)),
    ]);
    frame.render_widget(Paragraph::new(category_line), chunks[1]);

    // Period
    let period_line = Line::from(vec![
        Span::styled("Period:   ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{}", app.current_period),
            Style::default().fg(Color::White),
        ),
    ]);
    frame.render_widget(Paragraph::new(period_line), chunks[2]);

    // Current amount
    let current_line = Line::from(vec![
        Span::styled("Current:  ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{}", state.current_amount),
            Style::default().fg(Color::White),
        ),
    ]);
    frame.render_widget(Paragraph::new(current_line), chunks[4]);

    // New amount input
    let new_amount_label = Line::from(Span::styled(
        "New amount:",
        Style::default().fg(Color::Cyan),
    ));
    frame.render_widget(Paragraph::new(new_amount_label), chunks[5]);

    // Input with cursor
    let mut input_spans = vec![Span::raw("$")];
    let (before, after) = state
        .amount_input
        .split_at(state.cursor_pos.min(state.amount_input.len()));

    input_spans.push(Span::styled(
        before.to_string(),
        Style::default().fg(Color::White),
    ));

    let cursor_char = after.chars().next().unwrap_or(' ');
    input_spans.push(Span::styled(
        cursor_char.to_string(),
        Style::default().fg(Color::Black).bg(Color::Cyan),
    ));

    if after.len() > 1 {
        input_spans.push(Span::styled(
            after[1..].to_string(),
            Style::default().fg(Color::White),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(input_spans)), chunks[6]);

    // Error message
    if let Some(ref error) = state.error_message {
        let error_line = Line::from(Span::styled(error.as_str(), Style::default().fg(Color::Red)));
        frame.render_widget(Paragraph::new(error_line), chunks[7]);
    }

    // Instructions
    let instructions = Line::from(vec![
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Save  "),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::raw(" Cancel  "),
        Span::styled("[Ctrl+U]", Style::default().fg(Color::Cyan)),
        Span::raw(" Clear"),
    ]);
    frame.render_widget(Paragraph::new(instructions), chunks[8]);
}

/// Handle key events for the edit budget dialog
pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::{KeyCode, KeyModifiers};

    match key.code {
        KeyCode::Esc => {
            app.edit_budget_state.reset();
            app.close_dialog();
            true
        }

        KeyCode::Enter => {
            // Parse and save the amount
            match app.edit_budget_state.parse_amount() {
                Ok(amount) => {
                    if let Some(category_id) = app.edit_budget_state.category_id {
                        let budget_service = BudgetService::new(app.storage);
                        match budget_service.assign_to_category(
                            category_id,
                            &app.current_period,
                            amount,
                        ) {
                            Ok(_) => {
                                let cat_name = app.edit_budget_state.category_name.clone();
                                app.set_status(format!(
                                    "Budget for '{}' set to {}",
                                    cat_name, amount
                                ));
                                app.edit_budget_state.reset();
                                app.close_dialog();
                            }
                            Err(e) => {
                                app.edit_budget_state.set_error(e.to_string());
                            }
                        }
                    }
                }
                Err(e) => {
                    app.edit_budget_state.set_error(e);
                }
            }
            true
        }

        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.edit_budget_state.clear_input();
            true
        }

        KeyCode::Char(c) => {
            app.edit_budget_state.insert_char(c);
            true
        }

        KeyCode::Backspace => {
            app.edit_budget_state.backspace();
            true
        }

        KeyCode::Left => {
            app.edit_budget_state.move_left();
            true
        }

        KeyCode::Right => {
            app.edit_budget_state.move_right();
            true
        }

        _ => false,
    }
}
