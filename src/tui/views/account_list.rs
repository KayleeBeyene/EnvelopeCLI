//! Account list view (main panel)
//!
//! Shows detailed account information

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::services::AccountService;
use crate::tui::app::{App, FocusedPanel};
use crate::tui::layout::MainPanelLayout;

/// Render the accounts view in the main panel
pub fn render_main(frame: &mut Frame, app: &mut App, area: Rect) {
    let layout = MainPanelLayout::new(area);

    // Render header
    render_header(frame, layout.header);

    // Render account table
    render_account_table(frame, app, layout.content);
}

/// Render header
fn render_header(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" All Accounts ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    frame.render_widget(block, area);
}

/// Render account table
fn render_account_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Main;
    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::White
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let account_service = AccountService::new(app.storage);
    let accounts = account_service
        .list_with_balances(app.show_archived)
        .unwrap_or_default();

    // Define column widths
    let widths = [
        ratatui::layout::Constraint::Length(20), // Name
        ratatui::layout::Constraint::Length(12), // Type
        ratatui::layout::Constraint::Length(14), // Balance
        ratatui::layout::Constraint::Length(14), // Cleared
        ratatui::layout::Constraint::Length(10), // Uncleared
        ratatui::layout::Constraint::Length(10), // On Budget
    ];

    // Header row
    let header = Row::new(vec![
        Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Type").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Balance").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Cleared").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Uncleared").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Budget").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().fg(Color::Yellow))
    .height(1);

    // Data rows
    let rows: Vec<Row> = accounts
        .iter()
        .map(|summary| {
            let balance_style = if summary.balance.is_negative() {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            };

            let cleared_style = if summary.cleared_balance.is_negative() {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            };

            Row::new(vec![
                Cell::from(summary.account.name.clone()),
                Cell::from(format!("{}", summary.account.account_type)),
                Cell::from(format!("{}", summary.balance)).style(balance_style),
                Cell::from(format!("{}", summary.cleared_balance)).style(cleared_style),
                Cell::from(format!("{}", summary.uncleared_count)),
                Cell::from(if summary.account.on_budget {
                    "Yes"
                } else {
                    "No"
                }),
            ])
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = TableState::default();
    state.select(Some(app.selected_account_index));

    frame.render_stateful_widget(table, area, &mut state);
}
