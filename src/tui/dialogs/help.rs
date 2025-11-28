//! Help dialog
//!
//! Shows contextual keyboard shortcuts

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::{ActiveView, App};
use crate::tui::layout::centered_rect;

/// Render the help dialog
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 70, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Build help text based on current view
    let help_lines = get_help_lines(app);

    let paragraph = Paragraph::new(help_lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Get help lines for the current context
fn get_help_lines(app: &App) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(vec![Span::styled(
            "Global Keys",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )]),
        Line::from(""),
        key_line("q", "Quit application"),
        key_line("?", "Show/hide help"),
        key_line(":", "Open command palette"),
        key_line("Tab", "Switch panel focus"),
        key_line("h/l", "Move focus left/right"),
        key_line("j/k", "Move selection up/down"),
        Line::from(""),
    ];

    // View-specific help
    match app.active_view {
        ActiveView::Accounts => {
            lines.push(Line::from(vec![Span::styled(
                "Account View",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )]));
            lines.push(Line::from(""));
            lines.push(key_line("Enter", "View transactions for account"));
            lines.push(key_line("1", "Switch to Accounts view"));
            lines.push(key_line("2", "Switch to Budget view"));
            lines.push(key_line("3", "Switch to Reports view"));
            lines.push(key_line("A", "Toggle archived accounts"));
        }
        ActiveView::Register => {
            lines.push(Line::from(vec![Span::styled(
                "Transaction Register",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )]));
            lines.push(Line::from(""));
            lines.push(key_line("a/n", "Add new transaction"));
            lines.push(key_line("e/Enter", "Edit transaction"));
            lines.push(key_line("c", "Toggle cleared status"));
            lines.push(key_line("Ctrl+d", "Delete transaction"));
            lines.push(key_line("g", "Go to top"));
            lines.push(key_line("G", "Go to bottom"));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Multi-Select Mode",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )]));
            lines.push(Line::from(""));
            lines.push(key_line("v", "Toggle multi-select mode"));
            lines.push(key_line("Space", "Toggle selection (in multi-select)"));
            lines.push(key_line("C", "Bulk categorize selected"));
        }
        ActiveView::Budget => {
            lines.push(Line::from(vec![Span::styled(
                "Budget View",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )]));
            lines.push(Line::from(""));
            lines.push(key_line("[/H", "Previous period"));
            lines.push(key_line("]/L", "Next period"));
            lines.push(key_line("m", "Move funds between categories"));
            lines.push(key_line("Enter", "Edit budget amount"));
        }
        ActiveView::Reports => {
            lines.push(Line::from(vec![Span::styled(
                "Reports View",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )]));
            lines.push(Line::from(""));
            lines.push(Line::from("Reports view coming soon!"));
        }
        ActiveView::Reconcile => {
            lines.push(Line::from(vec![Span::styled(
                "Reconciliation",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )]));
            lines.push(Line::from(""));
            lines.push(key_line("Tab", "Switch between header and transactions"));
            lines.push(key_line("Space", "Toggle cleared status"));
            lines.push(key_line("Enter", "Start reconciliation / Complete"));
            lines.push(key_line("j/k", "Navigate transactions"));
            lines.push(key_line("Esc", "Cancel reconciliation"));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Press any key to close",
        Style::default().fg(Color::DarkGray),
    )]));

    lines
}

/// Create a formatted key line
fn key_line(key: &str, description: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{:>12}", key), Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(description.to_string(), Style::default().fg(Color::White)),
    ])
}
