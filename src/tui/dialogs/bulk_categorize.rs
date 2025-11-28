//! Bulk categorize dialog
//!
//! Apply category to multiple selected transactions

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::models::CategoryId;
use crate::services::CategoryService;
use crate::tui::app::App;
use crate::tui::layout::centered_rect;

/// State for the bulk categorize dialog
#[derive(Debug, Clone, Default)]
pub struct BulkCategorizeState {
    /// Selected category
    pub selected_category: Option<CategoryId>,
    /// Index in the category list
    pub category_list_index: usize,
    /// Search/filter input
    pub search_input: String,
    /// Search cursor position
    pub search_cursor: usize,
    /// Error message
    pub error_message: Option<String>,
    /// Success message
    pub success_message: Option<String>,
}

impl BulkCategorizeState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the state
    pub fn reset(&mut self) {
        *self = Self::default();
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
        self.search_input.insert(self.search_cursor, c);
        self.search_cursor += 1;
        // Reset selection when typing
        self.category_list_index = 0;
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.search_cursor > 0 {
            self.search_cursor -= 1;
            self.search_input.remove(self.search_cursor);
            // Reset selection when typing
            self.category_list_index = 0;
        }
    }

    /// Clear search
    pub fn clear_search(&mut self) {
        self.search_input.clear();
        self.search_cursor = 0;
        self.category_list_index = 0;
    }
}

/// Render the bulk categorize dialog
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(55, 60, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let count = app.selected_transactions.len();

    let block = Block::default()
        .title(format!(
            " Categorize {} Transaction{} ",
            count,
            if count == 1 { "" } else { "s" }
        ))
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
            Constraint::Length(1), // Search label
            Constraint::Length(1), // Search input
            Constraint::Length(1), // Spacer
            Constraint::Min(6),    // Category list
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Error/success
            Constraint::Length(1), // Hints
        ])
        .split(inner);

    // Get categories
    let category_service = CategoryService::new(app.storage);
    let all_categories = category_service.list_categories().unwrap_or_default();

    // Filter categories by search
    let search = app.bulk_categorize_state.search_input.to_lowercase();
    let filtered_categories: Vec<_> = all_categories
        .iter()
        .filter(|c| search.is_empty() || c.name.to_lowercase().contains(&search))
        .collect();

    // Search input
    render_search_field(
        frame,
        &app.bulk_categorize_state.search_input,
        app.bulk_categorize_state.search_cursor,
        chunks[0],
        chunks[1],
    );

    // Category list
    render_category_list(
        frame,
        &filtered_categories,
        app.bulk_categorize_state.selected_category,
        app.bulk_categorize_state.category_list_index,
        chunks[3],
    );

    // Error/success message
    if let Some(ref error) = app.bulk_categorize_state.error_message {
        let error_line = Line::from(Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        ));
        frame.render_widget(Paragraph::new(error_line), chunks[5]);
    } else if let Some(ref success) = app.bulk_categorize_state.success_message {
        let success_line = Line::from(Span::styled(
            success.as_str(),
            Style::default().fg(Color::Green),
        ));
        frame.render_widget(Paragraph::new(success_line), chunks[5]);
    }

    // Hints
    let hints = Line::from(vec![
        Span::styled("[↑↓]", Style::default().fg(Color::Yellow)),
        Span::raw(" Select  "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Apply  "),
        Span::styled("[Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Cancel"),
    ]);
    frame.render_widget(Paragraph::new(hints), chunks[6]);
}

/// Render the search field
fn render_search_field(
    frame: &mut Frame,
    search: &str,
    cursor: usize,
    label_area: Rect,
    input_area: Rect,
) {
    // Label
    let label = Line::from(Span::styled(
        "Search categories:",
        Style::default().fg(Color::Cyan),
    ));
    frame.render_widget(Paragraph::new(label), label_area);

    // Input with cursor
    let mut spans = vec![Span::raw("  ")];

    let cursor_pos = cursor.min(search.len());
    let (before, after) = search.split_at(cursor_pos);

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

    if search.is_empty() {
        spans.push(Span::styled(
            " (type to filter)",
            Style::default().fg(Color::Yellow),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), input_area);
}

/// Render the category list
fn render_category_list(
    frame: &mut Frame,
    categories: &[&crate::models::Category],
    selected: Option<CategoryId>,
    list_index: usize,
    area: Rect,
) {
    if categories.is_empty() {
        let text =
            Paragraph::new("No matching categories").style(Style::default().fg(Color::Yellow));
        frame.render_widget(text, area);
        return;
    }

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

    frame.render_stateful_widget(list, area, &mut state);
}

/// Handle key events for the bulk categorize dialog
pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::KeyCode;

    // Get filtered categories for index bounds
    let category_service = CategoryService::new(app.storage);
    let all_categories = category_service.list_categories().unwrap_or_default();
    let search = app.bulk_categorize_state.search_input.to_lowercase();
    let filtered: Vec<_> = all_categories
        .iter()
        .filter(|c| search.is_empty() || c.name.to_lowercase().contains(&search))
        .collect();
    let cat_count = filtered.len();

    match key.code {
        KeyCode::Esc => {
            app.bulk_categorize_state.reset();
            app.close_dialog();
            return true;
        }

        KeyCode::Enter => {
            // Apply the selected category
            if cat_count > 0 {
                let idx = app
                    .bulk_categorize_state
                    .category_list_index
                    .min(cat_count.saturating_sub(1));
                if let Some(cat) = filtered.get(idx) {
                    execute_bulk_categorize(app, cat.id);
                }
            } else {
                app.bulk_categorize_state.set_error("No category selected");
            }
            return true;
        }

        KeyCode::Up | KeyCode::Char('k') => {
            if app.bulk_categorize_state.category_list_index > 0 {
                app.bulk_categorize_state.category_list_index -= 1;
            }
            return true;
        }

        KeyCode::Down | KeyCode::Char('j') => {
            if app.bulk_categorize_state.category_list_index < cat_count.saturating_sub(1) {
                app.bulk_categorize_state.category_list_index += 1;
            }
            return true;
        }

        KeyCode::Char(c) => {
            app.bulk_categorize_state.clear_error();
            app.bulk_categorize_state.insert_char(c);
            return true;
        }

        KeyCode::Backspace => {
            app.bulk_categorize_state.clear_error();
            app.bulk_categorize_state.backspace();
            return true;
        }

        KeyCode::Delete => {
            app.bulk_categorize_state.clear_search();
            return true;
        }

        _ => {}
    }

    false
}

/// Execute the bulk categorize operation
fn execute_bulk_categorize(app: &mut App, category_id: CategoryId) {
    let transaction_ids = app.selected_transactions.clone();

    if transaction_ids.is_empty() {
        app.bulk_categorize_state
            .set_error("No transactions selected");
        return;
    }

    let mut success_count = 0;
    let mut error_count = 0;

    for txn_id in &transaction_ids {
        match app.storage.transactions.get(*txn_id) {
            Ok(Some(mut txn)) => {
                // Skip transfers (they shouldn't be categorized)
                if txn.is_transfer() {
                    continue;
                }

                // Update category
                txn.category_id = Some(category_id);
                txn.updated_at = chrono::Utc::now();

                if app.storage.transactions.upsert(txn).is_ok() {
                    success_count += 1;
                } else {
                    error_count += 1;
                }
            }
            _ => {
                error_count += 1;
            }
        }
    }

    // Save all changes
    if let Err(e) = app.storage.transactions.save() {
        app.bulk_categorize_state
            .set_error(format!("Failed to save: {}", e));
        return;
    }

    // Get category name for message
    let category_service = CategoryService::new(app.storage);
    let category_name = category_service
        .get_category(category_id)
        .ok()
        .flatten()
        .map(|c| c.name)
        .unwrap_or_else(|| "Unknown".into());

    // Clear selections
    app.selected_transactions.clear();
    app.multi_select_mode = false;

    // Close dialog and show status
    if error_count > 0 {
        app.set_status(format!(
            "Categorized {} transactions as '{}' ({} errors)",
            success_count, category_name, error_count
        ));
    } else {
        app.set_status(format!(
            "Categorized {} transaction{} as '{}'",
            success_count,
            if success_count == 1 { "" } else { "s" },
            category_name
        ));
    }

    app.bulk_categorize_state.reset();
    app.close_dialog();
}
