//! Command palette dialog
//!
//! Provides fuzzy search for commands

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::commands::COMMANDS;
use crate::tui::layout::centered_rect_fixed;

/// Render the command palette
pub fn render(frame: &mut Frame, app: &mut App) {
    let width = 60;
    let height = 20;
    let area = centered_rect_fixed(width, height, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Command Palette ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(block, area);

    // Input area
    let input_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width - 2,
        height: 1,
    };

    let input_line = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::styled(app.command_input.clone(), Style::default().fg(Color::White)),
        Span::styled("_", Style::default().fg(Color::Cyan)), // Cursor
    ]);

    frame.render_widget(Paragraph::new(input_line), input_area);

    // Results area
    let results_area = Rect {
        x: area.x + 1,
        y: area.y + 3,
        width: area.width - 2,
        height: area.height - 4,
    };

    // Filter commands based on input
    let filtered_commands: Vec<(usize, &crate::tui::commands::Command)> = COMMANDS
        .iter()
        .enumerate()
        .filter(|(_, cmd)| {
            if app.command_input.is_empty() {
                true
            } else {
                let query = app.command_input.to_lowercase();
                cmd.name.to_lowercase().contains(&query)
                    || cmd.description.to_lowercase().contains(&query)
            }
        })
        .collect();

    if filtered_commands.is_empty() {
        let text =
            Paragraph::new("No matching commands").style(Style::default().fg(Color::Yellow));
        frame.render_widget(text, results_area);
        return;
    }

    // Build list items
    let items: Vec<ListItem> = filtered_commands
        .iter()
        .map(|(_, cmd)| {
            let line = Line::from(vec![
                Span::styled(
                    format!("{:<20}", cmd.name),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(" "),
                Span::styled(cmd.description, Style::default().fg(Color::Yellow)),
            ]);
            ListItem::new(line)
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
    let selected = app
        .selected_command_index
        .min(filtered_commands.len().saturating_sub(1));
    state.select(Some(selected));

    frame.render_stateful_widget(list, results_area, &mut state);
}
