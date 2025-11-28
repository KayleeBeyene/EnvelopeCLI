//! Category entry dialog
//!
//! Modal dialog for adding new budget categories with form validation,
//! group selection, and save/cancel functionality.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::models::{Category, CategoryGroupId};
use crate::services::CategoryService;
use crate::tui::app::App;
use crate::tui::layout::centered_rect;
use crate::tui::widgets::input::TextInput;

/// Which field is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CategoryField {
    #[default]
    Name,
    Group,
}

/// State for the category form dialog
#[derive(Debug, Clone)]
pub struct CategoryFormState {
    /// Name input
    pub name_input: TextInput,

    /// Selected group index
    pub selected_group_index: usize,

    /// Available groups (cached)
    pub groups: Vec<(CategoryGroupId, String)>,

    /// Currently focused field
    pub focused_field: CategoryField,

    /// Error message to display
    pub error_message: Option<String>,

    /// Category ID being edited (None for new category)
    pub editing_id: Option<crate::models::CategoryId>,
}

impl Default for CategoryFormState {
    fn default() -> Self {
        Self::new()
    }
}

impl CategoryFormState {
    /// Create a new form state with default values
    pub fn new() -> Self {
        Self {
            name_input: TextInput::new()
                .label("Name")
                .placeholder("Category name (e.g., Groceries, Rent)"),
            selected_group_index: 0,
            groups: Vec::new(),
            focused_field: CategoryField::Name,
            error_message: None,
            editing_id: None,
        }
    }

    /// Initialize the form with available groups
    pub fn init_with_groups(&mut self, groups: Vec<(CategoryGroupId, String)>) {
        self.groups = groups;
        self.selected_group_index = 0;
        self.name_input = TextInput::new()
            .label("Name")
            .placeholder("Category name (e.g., Groceries, Rent)");
        self.focused_field = CategoryField::Name;
        self.error_message = None;
        self.editing_id = None;
    }

    /// Initialize the form for editing an existing category
    pub fn init_for_edit(&mut self, category: &Category, groups: Vec<(CategoryGroupId, String)>) {
        self.groups = groups;
        self.editing_id = Some(category.id);

        // Set name input with existing value
        self.name_input = TextInput::new()
            .label("Name")
            .placeholder("Category name (e.g., Groceries, Rent)")
            .content(&category.name);

        // Find and select the current group
        self.selected_group_index = self
            .groups
            .iter()
            .position(|(id, _)| *id == category.group_id)
            .unwrap_or(0);

        self.focused_field = CategoryField::Name;
        self.error_message = None;
    }

    /// Set the focused field
    pub fn set_focus(&mut self, field: CategoryField) {
        self.focused_field = field;
    }

    /// Move to next field
    pub fn next_field(&mut self) {
        self.focused_field = match self.focused_field {
            CategoryField::Name => CategoryField::Group,
            CategoryField::Group => CategoryField::Name,
        };
    }

    /// Move to previous field
    pub fn prev_field(&mut self) {
        self.focused_field = match self.focused_field {
            CategoryField::Name => CategoryField::Group,
            CategoryField::Group => CategoryField::Name,
        };
    }

    /// Select next group
    pub fn next_group(&mut self) {
        if !self.groups.is_empty() {
            self.selected_group_index = (self.selected_group_index + 1) % self.groups.len();
        }
    }

    /// Select previous group
    pub fn prev_group(&mut self) {
        if !self.groups.is_empty() {
            self.selected_group_index = if self.selected_group_index == 0 {
                self.groups.len() - 1
            } else {
                self.selected_group_index - 1
            };
        }
    }

    /// Get selected group ID
    pub fn selected_group_id(&self) -> Option<CategoryGroupId> {
        self.groups
            .get(self.selected_group_index)
            .map(|(id, _)| *id)
    }

    /// Get selected group name
    pub fn selected_group_name(&self) -> Option<&str> {
        self.groups
            .get(self.selected_group_index)
            .map(|(_, name)| name.as_str())
    }

    /// Validate the form and return any error
    pub fn validate(&self) -> Result<(), String> {
        let name = self.name_input.value().trim();
        if name.is_empty() {
            return Err("Category name is required".to_string());
        }
        if name.len() > 50 {
            return Err("Category name too long (max 50 chars)".to_string());
        }
        if self.groups.is_empty() {
            return Err("No category groups available. Create a group first (Shift+A)".to_string());
        }
        Ok(())
    }

    /// Build a Category from the form state
    pub fn build_category(&self) -> Result<Category, String> {
        self.validate()?;
        let name = self.name_input.value().trim().to_string();
        let group_id = self
            .selected_group_id()
            .ok_or_else(|| "No group selected".to_string())?;
        Ok(Category::new(&name, group_id))
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

/// Render the category dialog
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(50, 30, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    // Choose title based on whether we're editing or adding
    let title = if app.category_form.editing_id.is_some() {
        " Edit Category "
    } else {
        " Add Category "
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
            Constraint::Length(1), // Name label
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Group label
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Error
            Constraint::Length(1), // Buttons
            Constraint::Min(0),    // Remaining
        ])
        .split(inner);

    // Extract values to avoid borrow conflicts
    let name_value = app.category_form.name_input.value().to_string();
    let name_cursor = app.category_form.name_input.cursor;
    let name_placeholder = app.category_form.name_input.placeholder.clone();
    let focused_field = app.category_form.focused_field;
    let error_message = app.category_form.error_message.clone();
    let group_name = app
        .category_form
        .selected_group_name()
        .unwrap_or("(no groups)")
        .to_string();

    // Render name field
    render_text_field(
        frame,
        chunks[0],
        "Name",
        &name_value,
        focused_field == CategoryField::Name,
        name_cursor,
        &name_placeholder,
    );

    // Render group selector
    render_selector_field(
        frame,
        chunks[2],
        "Group",
        &group_name,
        focused_field == CategoryField::Group,
    );

    // Render error message if any
    if let Some(ref error) = error_message {
        let error_line = Line::from(Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        ));
        frame.render_widget(Paragraph::new(error_line), chunks[4]);
    }

    // Render buttons/hints
    let hints = Line::from(vec![
        Span::styled("[Tab]", Style::default().fg(Color::Yellow)),
        Span::raw(" Next  "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Save  "),
        Span::styled("[Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Cancel"),
    ]);
    frame.render_widget(Paragraph::new(hints), chunks[5]);
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

/// Render a selector field (for group selection)
fn render_selector_field(frame: &mut Frame, area: Rect, label: &str, value: &str, focused: bool) {
    let label_style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let value_style = if focused {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let hint = if focused { " ◀ j/k ▶" } else { "" };

    let line = Line::from(vec![
        Span::styled(format!("{}: ", label), label_style),
        Span::styled(format!(" {} ", value), value_style),
        Span::styled(hint, Style::default().fg(Color::Yellow)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

/// Handle key input for the category dialog
pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Esc => {
            app.close_dialog();
            return true;
        }

        KeyCode::Tab | KeyCode::Down if app.category_form.focused_field == CategoryField::Name => {
            app.category_form.next_field();
            return true;
        }

        KeyCode::BackTab | KeyCode::Up
            if app.category_form.focused_field == CategoryField::Group =>
        {
            app.category_form.prev_field();
            return true;
        }

        KeyCode::Enter => {
            // Try to save
            if let Err(e) = save_category(app) {
                app.category_form.set_error(e);
            }
            return true;
        }

        _ => {}
    }

    // Field-specific handling
    match app.category_form.focused_field {
        CategoryField::Name => handle_name_input(app, key),
        CategoryField::Group => handle_group_selector(app, key),
    }
}

/// Handle input for the name field
fn handle_name_input(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::KeyCode;

    let form = &mut app.category_form;

    match key.code {
        KeyCode::Backspace => {
            form.clear_error();
            form.name_input.backspace();
            true
        }

        KeyCode::Delete => {
            form.clear_error();
            form.name_input.delete();
            true
        }

        KeyCode::Left => {
            form.name_input.move_left();
            true
        }

        KeyCode::Right => {
            form.name_input.move_right();
            true
        }

        KeyCode::Home => {
            form.name_input.move_start();
            true
        }

        KeyCode::End => {
            form.name_input.move_end();
            true
        }

        KeyCode::Char(c) => {
            form.clear_error();
            form.name_input.insert(c);
            true
        }

        _ => false,
    }
}

/// Handle input for the group selector
fn handle_group_selector(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::KeyCode;

    let form = &mut app.category_form;

    match key.code {
        KeyCode::Char('j') | KeyCode::Right => {
            form.next_group();
            true
        }

        KeyCode::Char('k') | KeyCode::Left => {
            form.prev_group();
            true
        }

        _ => false,
    }
}

/// Save the category
fn save_category(app: &mut App) -> Result<(), String> {
    // Validate form
    app.category_form.validate()?;

    let name = app.category_form.name_input.value().trim().to_string();
    let group_id = app
        .category_form
        .selected_group_id()
        .ok_or_else(|| "No group selected".to_string())?;

    let category_service = CategoryService::new(app.storage);

    if let Some(category_id) = app.category_form.editing_id {
        // Update existing category
        category_service
            .update_category(category_id, Some(&name), None, false)
            .map_err(|e| e.to_string())?;

        // If group changed, move the category
        if let Ok(Some(cat)) = app.storage.categories.get_category(category_id) {
            if cat.group_id != group_id {
                category_service
                    .move_category(category_id, group_id)
                    .map_err(|e| e.to_string())?;
            }
        }

        // Close dialog
        app.close_dialog();
        app.set_status(format!("Category '{}' updated", name));
    } else {
        // Create new category
        category_service
            .create_category(&name, group_id)
            .map_err(|e| e.to_string())?;

        // Close dialog
        app.close_dialog();
        app.set_status(format!("Category '{}' created", name));
    }

    Ok(())
}
