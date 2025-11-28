//! Reconcile start dialog
//!
//! Dialog to enter statement date and balance to begin reconciliation.

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::layout::centered_rect_fixed;

/// State for the reconcile start dialog
#[derive(Debug, Clone, Default)]
pub struct ReconcileStartState {
    /// Statement date input
    pub date_input: String,
    /// Statement balance input
    pub balance_input: String,
    /// Active field (0=date, 1=balance)
    pub active_field: usize,
}

impl ReconcileStartState {
    pub fn new() -> Self {
        Self {
            date_input: chrono::Local::now().format("%Y-%m-%d").to_string(),
            balance_input: String::new(),
            active_field: 0,
        }
    }
}

/// Render the reconcile start dialog
pub fn render(frame: &mut Frame, app: &App) {
    let area = centered_rect_fixed(50, 12, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Start Reconciliation ")
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
            Constraint::Length(2), // Date label + input
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Balance label + input
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Instructions
        ])
        .split(inner);

    let state = &app.reconcile_start_state;

    // Date field
    let date_style = if state.active_field == 0 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let date_label = if state.active_field == 0 {
        Span::styled("Statement Date: ", Style::default().fg(Color::Yellow))
    } else {
        Span::styled("Statement Date: ", Style::default().fg(Color::White))
    };

    let date_text = Paragraph::new(vec![
        Line::from(date_label),
        Line::from(Span::styled(&state.date_input, date_style)),
    ]);
    frame.render_widget(date_text, chunks[1]);

    // Balance field
    let balance_style = if state.active_field == 1 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let balance_label = if state.active_field == 1 {
        Span::styled("Statement Balance: ", Style::default().fg(Color::Yellow))
    } else {
        Span::styled("Statement Balance: ", Style::default().fg(Color::White))
    };

    let balance_display = if state.balance_input.is_empty() {
        "Enter balance...".to_string()
    } else {
        state.balance_input.clone()
    };

    let balance_text = Paragraph::new(vec![
        Line::from(balance_label),
        Line::from(Span::styled(balance_display, balance_style)),
    ]);
    frame.render_widget(balance_text, chunks[3]);

    // Instructions
    let instructions = Paragraph::new(Line::from(vec![
        Span::styled("[Tab]", Style::default().fg(Color::Green)),
        Span::raw(" Switch field  "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Start  "),
        Span::styled("[Esc]", Style::default().fg(Color::White)),
        Span::raw(" Cancel"),
    ]));
    frame.render_widget(instructions, chunks[5]);
}

/// Handle key input for the reconcile start dialog
pub fn handle_key(app: &mut App, key: crossterm::event::KeyCode) -> bool {
    use crossterm::event::KeyCode;

    let state = &mut app.reconcile_start_state;

    match key {
        KeyCode::Tab => {
            state.active_field = (state.active_field + 1) % 2;
            true
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if state.active_field > 0 {
                state.active_field -= 1;
            }
            true
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.active_field < 1 {
                state.active_field += 1;
            }
            true
        }
        KeyCode::Char(c) => {
            if state.active_field == 0 {
                state.date_input.push(c);
            } else {
                state.balance_input.push(c);
            }
            true
        }
        KeyCode::Backspace => {
            if state.active_field == 0 {
                state.date_input.pop();
            } else {
                state.balance_input.pop();
            }
            true
        }
        _ => false,
    }
}
