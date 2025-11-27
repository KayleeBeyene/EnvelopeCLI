//! Adjustment transaction dialog
//!
//! Dialog to confirm creating an adjustment transaction during reconciliation.

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::models::{CategoryId, Money};
use crate::tui::app::App;
use crate::tui::layout::centered_rect_fixed;

/// State for the adjustment dialog
#[derive(Debug, Clone, Default)]
pub struct AdjustmentDialogState {
    /// The adjustment amount needed
    pub adjustment_amount: Money,
    /// Category input for the adjustment
    pub category_input: String,
    /// Selected category ID
    pub selected_category: Option<CategoryId>,
    /// List of categories for selection
    pub categories: Vec<(CategoryId, String)>,
    /// Selected category index
    pub selected_index: usize,
    /// Whether in category selection mode
    pub selecting_category: bool,
}

impl AdjustmentDialogState {
    pub fn new(adjustment_amount: Money) -> Self {
        Self {
            adjustment_amount,
            category_input: String::new(),
            selected_category: None,
            categories: Vec::new(),
            selected_index: 0,
            selecting_category: false,
        }
    }

    /// Load categories from storage
    pub fn load_categories(&mut self, categories: Vec<(CategoryId, String)>) {
        self.categories = categories;
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selecting_category && self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selecting_category && self.selected_index < self.categories.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Get filtered categories based on input
    pub fn filtered_categories(&self) -> Vec<&(CategoryId, String)> {
        if self.category_input.is_empty() {
            self.categories.iter().collect()
        } else {
            let search = self.category_input.to_lowercase();
            self.categories
                .iter()
                .filter(|(_, name)| name.to_lowercase().contains(&search))
                .collect()
        }
    }

    /// Select the current category
    pub fn select_current(&mut self) {
        // Clone to avoid borrow issues
        let selection: Option<(CategoryId, String)> = {
            let filtered = self.filtered_categories();
            filtered.get(self.selected_index).map(|(id, name)| (*id, name.clone()))
        };

        if let Some((id, name)) = selection {
            self.selected_category = Some(id);
            self.category_input = name;
            self.selecting_category = false;
        }
    }
}

/// Render the adjustment dialog
pub fn render(frame: &mut Frame, app: &App) {
    let area = centered_rect_fixed(60, 14, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let state = &app.adjustment_dialog_state;

    let block = Block::default()
        .title(" Create Adjustment Transaction ")
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Spacer
            Constraint::Length(2),  // Message
            Constraint::Length(2),  // Amount
            Constraint::Length(2),  // Category input
            Constraint::Length(3),  // Category list (if selecting)
            Constraint::Length(2),  // Instructions
        ])
        .split(inner);

    // Message
    let message = Paragraph::new(Line::from(vec![
        Span::styled(
            "Your cleared balance doesn't match the statement balance.",
            Style::default().fg(Color::White),
        ),
    ]));
    frame.render_widget(message, chunks[1]);

    // Amount
    let amount_style = if state.adjustment_amount.is_negative() {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    };

    let amount_text = Paragraph::new(Line::from(vec![
        Span::styled("Adjustment needed: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", state.adjustment_amount), amount_style),
    ]));
    frame.render_widget(amount_text, chunks[2]);

    // Category input
    let category_style = if state.selecting_category {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let category_display = if state.category_input.is_empty() {
        "(Optional) Enter category...".to_string()
    } else {
        state.category_input.clone()
    };

    let category_text = Paragraph::new(Line::from(vec![
        Span::styled("Category: ", Style::default().fg(Color::DarkGray)),
        Span::styled(category_display, category_style),
    ]));
    frame.render_widget(category_text, chunks[3]);

    // Category selection list (if selecting)
    if state.selecting_category {
        let filtered = state.filtered_categories();
        let items: Vec<Line> = filtered
            .iter()
            .enumerate()
            .take(3)
            .map(|(i, (_, name))| {
                let style = if i == state.selected_index {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                Line::from(Span::styled(format!("  {}", name), style))
            })
            .collect();

        let list = Paragraph::new(items);
        frame.render_widget(list, chunks[4]);
    }

    // Instructions
    let instructions = Paragraph::new(Line::from(vec![
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Create  "),
        Span::styled("[Tab]", Style::default().fg(Color::Cyan)),
        Span::raw(" Select category  "),
        Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
        Span::raw(" Cancel"),
    ]));
    frame.render_widget(instructions, chunks[5]);
}

/// Handle key input for the adjustment dialog
pub fn handle_key(app: &mut App, key: crossterm::event::KeyCode) -> bool {
    use crossterm::event::KeyCode;

    let state = &mut app.adjustment_dialog_state;

    match key {
        KeyCode::Tab => {
            state.selecting_category = !state.selecting_category;
            true
        }
        KeyCode::Up | KeyCode::Char('k') if state.selecting_category => {
            state.move_up();
            true
        }
        KeyCode::Down | KeyCode::Char('j') if state.selecting_category => {
            state.move_down();
            true
        }
        KeyCode::Enter if state.selecting_category => {
            state.select_current();
            true
        }
        KeyCode::Char(c) if !state.selecting_category => {
            state.category_input.push(c);
            state.selecting_category = true;
            state.selected_index = 0;
            true
        }
        KeyCode::Backspace if !state.selecting_category => {
            state.category_input.pop();
            true
        }
        _ => false,
    }
}
