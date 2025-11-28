//! Move funds dialog
//!
//! Transfer budget between categories

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::models::{CategoryId, Money};
use crate::services::{BudgetService, CategoryService};
use crate::tui::app::App;
use crate::tui::layout::centered_rect;

/// Which field is focused in the move funds dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MoveFundsField {
    #[default]
    FromCategory,
    ToCategory,
    Amount,
}

impl MoveFundsField {
    pub fn next(self) -> Self {
        match self {
            Self::FromCategory => Self::ToCategory,
            Self::ToCategory => Self::Amount,
            Self::Amount => Self::FromCategory,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::FromCategory => Self::Amount,
            Self::ToCategory => Self::FromCategory,
            Self::Amount => Self::ToCategory,
        }
    }
}

/// State for the move funds dialog
#[derive(Debug, Clone, Default)]
pub struct MoveFundsState {
    /// Currently focused field
    pub focused_field: MoveFundsField,
    /// Selected source category
    pub from_category: Option<CategoryId>,
    /// Selected destination category
    pub to_category: Option<CategoryId>,
    /// Index in the from category list
    pub from_list_index: usize,
    /// Index in the to category list
    pub to_list_index: usize,
    /// Amount to move (as string for editing)
    pub amount_input: String,
    /// Amount cursor position
    pub amount_cursor: usize,
    /// Error message
    pub error_message: Option<String>,
    /// Success message
    pub success_message: Option<String>,
}

impl MoveFundsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the state
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Move to next field
    pub fn next_field(&mut self) {
        self.focused_field = self.focused_field.next();
    }

    /// Move to previous field
    pub fn prev_field(&mut self) {
        self.focused_field = self.focused_field.prev();
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Set error message
    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
        self.success_message = None;
    }

    /// Set success message
    pub fn set_success(&mut self, msg: impl Into<String>) {
        self.success_message = Some(msg.into());
        self.error_message = None;
    }

    /// Insert character at cursor
    pub fn insert_char(&mut self, c: char) {
        if c.is_ascii_digit() || c == '.' || c == '-' {
            self.amount_input.insert(self.amount_cursor, c);
            self.amount_cursor += 1;
        }
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.amount_cursor > 0 {
            self.amount_cursor -= 1;
            self.amount_input.remove(self.amount_cursor);
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.amount_cursor > 0 {
            self.amount_cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        if self.amount_cursor < self.amount_input.len() {
            self.amount_cursor += 1;
        }
    }

    /// Validate the form
    pub fn validate(&self) -> Result<(CategoryId, CategoryId, Money), String> {
        let from = self.from_category.ok_or("Select a source category")?;
        let to = self.to_category.ok_or("Select a destination category")?;

        if from == to {
            return Err("Source and destination must be different".into());
        }

        if self.amount_input.trim().is_empty() {
            return Err("Enter an amount to move".into());
        }

        let amount = Money::parse(&self.amount_input).map_err(|_| "Invalid amount format")?;

        if amount.is_zero() {
            return Err("Amount must be greater than zero".into());
        }

        if amount.is_negative() {
            return Err("Amount must be positive".into());
        }

        Ok((from, to, amount))
    }
}

/// Render the move funds dialog
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 70, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Move Funds ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(block, area);

    // Inner area
    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    // Layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // From label
            Constraint::Length(6), // From list
            Constraint::Length(1), // To label
            Constraint::Length(6), // To list
            Constraint::Length(1), // Amount label
            Constraint::Length(1), // Amount input
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Error/success
            Constraint::Length(1), // Hints
            Constraint::Min(0),    // Remaining
        ])
        .split(inner);

    // Get categories
    let category_service = CategoryService::new(app.storage);
    let categories = category_service.list_categories().unwrap_or_default();

    // Title
    let title = Line::from(Span::styled(
        "Move Budget Between Categories",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(title), chunks[0]);

    // From category
    let from_focused = app.move_funds_state.focused_field == MoveFundsField::FromCategory;
    render_category_field(
        frame,
        &categories,
        "From:",
        app.move_funds_state.from_category,
        app.move_funds_state.from_list_index,
        from_focused,
        chunks[2],
        chunks[3],
    );

    // To category
    let to_focused = app.move_funds_state.focused_field == MoveFundsField::ToCategory;
    render_category_field(
        frame,
        &categories,
        "To:",
        app.move_funds_state.to_category,
        app.move_funds_state.to_list_index,
        to_focused,
        chunks[4],
        chunks[5],
    );

    // Amount
    let amount_focused = app.move_funds_state.focused_field == MoveFundsField::Amount;
    render_amount_field(
        frame,
        &app.move_funds_state.amount_input,
        app.move_funds_state.amount_cursor,
        amount_focused,
        chunks[6],
        chunks[7],
    );

    // Error/success message
    if let Some(ref error) = app.move_funds_state.error_message {
        let error_line = Line::from(Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        ));
        frame.render_widget(Paragraph::new(error_line), chunks[9]);
    } else if let Some(ref success) = app.move_funds_state.success_message {
        let success_line = Line::from(Span::styled(
            success.as_str(),
            Style::default().fg(Color::Green),
        ));
        frame.render_widget(Paragraph::new(success_line), chunks[9]);
    }

    // Hints
    let hints = Line::from(vec![
        Span::styled("[Tab]", Style::default().fg(Color::White)),
        Span::raw(" Next  "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Move  "),
        Span::styled("[Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Cancel"),
    ]);
    frame.render_widget(Paragraph::new(hints), chunks[10]);
}

/// Render a category selection field
#[allow(clippy::too_many_arguments)]
fn render_category_field(
    frame: &mut Frame,
    categories: &[crate::models::Category],
    label: &str,
    selected: Option<CategoryId>,
    list_index: usize,
    focused: bool,
    label_area: Rect,
    list_area: Rect,
) {
    let label_style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    // Render label with selected category name
    let selected_name = if let Some(cat_id) = selected {
        categories
            .iter()
            .find(|c| c.id == cat_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Unknown".into())
    } else {
        "(none)".into()
    };

    let label_line = Line::from(vec![
        Span::styled(format!("{:>8} ", label), label_style),
        Span::styled(
            selected_name,
            if selected.is_some() {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::White)
            },
        ),
    ]);
    frame.render_widget(Paragraph::new(label_line), label_area);

    // Render category list if focused
    if focused {
        let items: Vec<ListItem> = categories
            .iter()
            .map(|cat| {
                let style = if Some(cat.id) == selected {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(Span::styled(format!("  {}", cat.name), style)))
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
        state.select(Some(list_index.min(categories.len().saturating_sub(1))));

        frame.render_stateful_widget(list, list_area, &mut state);
    } else {
        // Show hint when not focused
        let hint = Paragraph::new("  (Tab to this field to select)")
            .style(Style::default().fg(Color::White));
        frame.render_widget(hint, list_area);
    }
}

/// Render the amount input field
fn render_amount_field(
    frame: &mut Frame,
    amount: &str,
    cursor: usize,
    focused: bool,
    label_area: Rect,
    input_area: Rect,
) {
    let label_style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let label_line = Line::from(Span::styled("  Amount: ", label_style));
    frame.render_widget(Paragraph::new(label_line), label_area);

    // Render input
    let display = if amount.is_empty() && !focused {
        "$0.00".to_string()
    } else {
        format!("${}", amount)
    };

    let mut spans = vec![Span::raw("         ")]; // Indent

    if focused {
        spans.push(Span::raw("$"));
        let cursor_in_amount = cursor;
        let (before, after) = amount.split_at(cursor_in_amount.min(amount.len()));

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
        spans.push(Span::styled(display, Style::default().fg(Color::Yellow)));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), input_area);
}

/// Handle key events for the move funds dialog
pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::{KeyCode, KeyModifiers};

    let category_service = CategoryService::new(app.storage);
    let categories = category_service.list_categories().unwrap_or_default();
    let cat_count = categories.len();

    match key.code {
        KeyCode::Esc => {
            app.move_funds_state.reset();
            app.close_dialog();
            return true;
        }

        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.move_funds_state.prev_field();
            } else {
                app.move_funds_state.next_field();
            }
            return true;
        }

        KeyCode::BackTab => {
            app.move_funds_state.prev_field();
            return true;
        }

        KeyCode::Enter => {
            // If in a category list, select the category
            match app.move_funds_state.focused_field {
                MoveFundsField::FromCategory => {
                    if let Some(cat) = categories.get(app.move_funds_state.from_list_index) {
                        app.move_funds_state.from_category = Some(cat.id);
                        app.move_funds_state.next_field();
                    }
                    return true;
                }
                MoveFundsField::ToCategory => {
                    if let Some(cat) = categories.get(app.move_funds_state.to_list_index) {
                        app.move_funds_state.to_category = Some(cat.id);
                        app.move_funds_state.next_field();
                    }
                    return true;
                }
                MoveFundsField::Amount => {
                    // Try to execute the move
                    execute_move(app);
                    return true;
                }
            }
        }

        KeyCode::Up | KeyCode::Char('k') => {
            match app.move_funds_state.focused_field {
                MoveFundsField::FromCategory => {
                    if app.move_funds_state.from_list_index > 0 {
                        app.move_funds_state.from_list_index -= 1;
                    }
                }
                MoveFundsField::ToCategory => {
                    if app.move_funds_state.to_list_index > 0 {
                        app.move_funds_state.to_list_index -= 1;
                    }
                }
                _ => {}
            }
            return true;
        }

        KeyCode::Down | KeyCode::Char('j') => {
            match app.move_funds_state.focused_field {
                MoveFundsField::FromCategory => {
                    if app.move_funds_state.from_list_index < cat_count.saturating_sub(1) {
                        app.move_funds_state.from_list_index += 1;
                    }
                }
                MoveFundsField::ToCategory => {
                    if app.move_funds_state.to_list_index < cat_count.saturating_sub(1) {
                        app.move_funds_state.to_list_index += 1;
                    }
                }
                _ => {}
            }
            return true;
        }

        KeyCode::Char(c) => {
            if app.move_funds_state.focused_field == MoveFundsField::Amount {
                app.move_funds_state.clear_error();
                app.move_funds_state.insert_char(c);
                return true;
            }
        }

        KeyCode::Backspace => {
            if app.move_funds_state.focused_field == MoveFundsField::Amount {
                app.move_funds_state.clear_error();
                app.move_funds_state.backspace();
                return true;
            }
        }

        KeyCode::Left => {
            if app.move_funds_state.focused_field == MoveFundsField::Amount {
                app.move_funds_state.move_left();
                return true;
            }
        }

        KeyCode::Right => {
            if app.move_funds_state.focused_field == MoveFundsField::Amount {
                app.move_funds_state.move_right();
                return true;
            }
        }

        _ => {}
    }

    false
}

/// Execute the move funds operation
fn execute_move(app: &mut App) {
    // Validate
    let (from_id, to_id, amount) = match app.move_funds_state.validate() {
        Ok(result) => result,
        Err(e) => {
            app.move_funds_state.set_error(e);
            return;
        }
    };

    // Execute move
    let budget_service = BudgetService::new(app.storage);
    match budget_service.move_between_categories(from_id, to_id, &app.current_period, amount) {
        Ok(()) => {
            // Get category names for message
            let category_service = CategoryService::new(app.storage);
            let from_name = category_service
                .get_category(from_id)
                .ok()
                .flatten()
                .map(|c| c.name)
                .unwrap_or_else(|| "Unknown".into());
            let to_name = category_service
                .get_category(to_id)
                .ok()
                .flatten()
                .map(|c| c.name)
                .unwrap_or_else(|| "Unknown".into());

            app.set_status(format!(
                "Moved {} from '{}' to '{}'",
                amount, from_name, to_name
            ));
            app.move_funds_state.reset();
            app.close_dialog();
        }
        Err(e) => {
            app.move_funds_state.set_error(e.to_string());
        }
    }
}
