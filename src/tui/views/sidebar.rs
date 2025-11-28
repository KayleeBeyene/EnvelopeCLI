//! Sidebar view
//!
//! Shows account list and view switcher

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::services::AccountService;
use crate::tui::app::{ActiveView, App, FocusedPanel};
use crate::tui::layout::SidebarLayout;

/// Render the sidebar
pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let layout = SidebarLayout::new(area);

    // Render header
    render_header(frame, layout.header);

    // Render account list
    render_accounts(frame, app, layout.accounts);

    // Render view switcher
    render_view_switcher(frame, app, layout.view_switcher);
}

/// Render sidebar header
fn render_header(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Envelope ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let version = Paragraph::new("v0.1.0")
        .block(block)
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(version, area);
}

/// Render account list
fn render_accounts(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Sidebar;

    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(" Accounts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    // Get accounts with balances
    let account_service = AccountService::new(app.storage);
    let accounts = account_service
        .list_with_balances(app.show_archived)
        .unwrap_or_default();

    if accounts.is_empty() {
        let text = Paragraph::new("No accounts")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, area);
        return;
    }

    // Build list items
    let items: Vec<ListItem> = accounts
        .iter()
        .map(|summary| {
            let balance_str = format!("{}", summary.balance);
            let balance_color = if summary.balance.is_negative() {
                Color::Red
            } else {
                Color::Green
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:<15}", truncate_string(&summary.account.name, 15)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:>12}", balance_str),
                    Style::default().fg(balance_color),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.selected_account_index));

    frame.render_stateful_widget(list, area, &mut state);
}

/// Render view switcher
fn render_view_switcher(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(" Views ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let views = [
        ("1", "Accounts", ActiveView::Accounts),
        ("2", "Budget", ActiveView::Budget),
        ("3", "Reports", ActiveView::Reports),
    ];

    let items: Vec<ListItem> = views
        .iter()
        .map(|(key, name, view)| {
            let style = if app.active_view == *view {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let indicator = if app.active_view == *view { "▶" } else { " " };

            let line = Line::from(vec![
                Span::styled(format!("{} ", indicator), style),
                Span::styled(format!("[{}] ", key), Style::default().fg(Color::Yellow)),
                Span::styled(*name, style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);

    frame.render_widget(list, area);
}

/// Truncate a string to a maximum length
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
