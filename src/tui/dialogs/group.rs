//! Category group entry dialog
//!
//! Modal dialog for adding new category groups with form validation
//! and save/cancel functionality.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::models::CategoryGroup;
use crate::services::CategoryService;
use crate::tui::app::App;
use crate::tui::layout::centered_rect;
use crate::tui::widgets::input::TextInput;

/// State for the group form dialog
#[derive(Debug, Clone)]
pub struct GroupFormState {
    /// Name input
    pub name_input: TextInput,

    /// Error message to display
    pub error_message: Option<String>,
}

impl Default for GroupFormState {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupFormState {
    /// Create a new form state with default values
    pub fn new() -> Self {
        Self {
            name_input: TextInput::new()
                .label("Name")
                .placeholder("Group name (e.g., Bills, Savings)"),
            error_message: None,
        }
    }

    /// Validate the form and return any error
    pub fn validate(&self) -> Result<(), String> {
        let name = self.name_input.value().trim();
        if name.is_empty() {
            return Err("Group name is required".to_string());
        }
        if name.len() > 50 {
            return Err("Group name too long (max 50 chars)".to_string());
        }
        Ok(())
    }

    /// Build a CategoryGroup from the form state
    pub fn build_group(&self) -> Result<CategoryGroup, String> {
        self.validate()?;
        let name = self.name_input.value().trim().to_string();
        Ok(CategoryGroup::new(name))
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

/// Render the group dialog
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 25, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Add Category Group ")
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
            Constraint::Length(1), // Error
            Constraint::Length(1), // Buttons
            Constraint::Min(0),    // Remaining
        ])
        .split(inner);

    // Extract values to avoid borrow conflicts
    let name_value = app.group_form.name_input.value().to_string();
    let name_cursor = app.group_form.name_input.cursor;
    let name_placeholder = app.group_form.name_input.placeholder.clone();
    let error_message = app.group_form.error_message.clone();

    // Render name field (always focused since it's the only field)
    render_text_field(
        frame,
        chunks[0],
        "Name",
        &name_value,
        true, // always focused
        name_cursor,
        &name_placeholder,
    );

    // Render error message if any
    if let Some(ref error) = error_message {
        let error_line = Line::from(Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        ));
        frame.render_widget(Paragraph::new(error_line), chunks[2]);
    }

    // Render buttons/hints
    let hints = Line::from(vec![
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Save  "),
        Span::styled("[Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Cancel"),
    ]);
    frame.render_widget(Paragraph::new(hints), chunks[3]);
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

/// Handle key input for the group dialog
pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::KeyCode;

    let form = &mut app.group_form;

    match key.code {
        KeyCode::Esc => {
            app.close_dialog();
            return true;
        }

        KeyCode::Enter => {
            // Try to save
            if let Err(e) = save_group(app) {
                app.group_form.set_error(e);
            }
            return true;
        }

        KeyCode::Backspace => {
            form.clear_error();
            form.name_input.backspace();
            return true;
        }

        KeyCode::Delete => {
            form.clear_error();
            form.name_input.delete();
            return true;
        }

        KeyCode::Left => {
            form.name_input.move_left();
            return true;
        }

        KeyCode::Right => {
            form.name_input.move_right();
            return true;
        }

        KeyCode::Home => {
            form.name_input.move_start();
            return true;
        }

        KeyCode::End => {
            form.name_input.move_end();
            return true;
        }

        KeyCode::Char(c) => {
            form.clear_error();
            form.name_input.insert(c);
            return true;
        }

        _ => {}
    }

    false
}

/// Save the group
fn save_group(app: &mut App) -> Result<(), String> {
    // Validate form
    app.group_form.validate()?;

    let name = app.group_form.name_input.value().trim().to_string();

    // Use CategoryService to create the group
    let category_service = CategoryService::new(app.storage);
    category_service
        .create_group(&name)
        .map_err(|e| e.to_string())?;

    // Save to disk
    app.storage.categories.save().map_err(|e| e.to_string())?;

    // Close dialog
    app.close_dialog();
    app.set_status(format!("Category group '{}' created", name));

    Ok(())
}
