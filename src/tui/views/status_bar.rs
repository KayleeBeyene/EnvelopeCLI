//! Status bar view
//!
//! Shows Available to Budget, current balance, and key hints

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::services::{AccountService, BudgetService};
use crate::tui::app::App;

/// Render the status bar
pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    // Get Available to Budget
    let budget_service = BudgetService::new(app.storage);
    let atb = budget_service
        .get_available_to_budget(&app.current_period)
        .unwrap_or_default();

    // Get total balance if an account is selected
    let balance_info = if let Some(account_id) = app.selected_account {
        let account_service = AccountService::new(app.storage);
        account_service.calculate_balance(account_id).ok()
    } else {
        None
    };

    // Build status line
    let mut spans = vec![];

    // Available to Budget
    let atb_color = if atb.is_negative() {
        Color::Red
    } else if atb.is_zero() {
        Color::Green
    } else {
        Color::Yellow
    };

    spans.push(Span::styled(" ATB: ", Style::default().fg(Color::White)));
    spans.push(Span::styled(
        format!("{}", atb),
        Style::default().fg(atb_color).add_modifier(Modifier::BOLD),
    ));

    // Separator
    spans.push(Span::raw(" │ "));

    // Period
    spans.push(Span::styled(
        format!("{}", app.current_period),
        Style::default().fg(Color::Cyan),
    ));

    // Selected account balance
    if let Some(balance) = balance_info {
        spans.push(Span::raw(" │ "));
        spans.push(Span::styled("Bal: ", Style::default().fg(Color::White)));
        let balance_color = if balance.is_negative() {
            Color::Red
        } else {
            Color::Green
        };
        spans.push(Span::styled(
            format!("{}", balance),
            Style::default().fg(balance_color),
        ));
    }

    // Status message if any
    if let Some(ref message) = app.status_message {
        spans.push(Span::raw(" │ "));
        spans.push(Span::styled(
            message.as_str(),
            Style::default().fg(Color::Yellow),
        ));
    }

    // Key hints (right-aligned)
    let hints = " q:Quit  ?:Help  / or ::Command ";

    // Calculate padding
    let left_len: usize = spans.iter().map(|s| s.content.len()).sum();
    let padding_len = area.width as usize - left_len - hints.len();
    let padding = " ".repeat(padding_len.max(1));

    spans.push(Span::raw(padding));
    spans.push(Span::styled(hints, Style::default().fg(Color::White)));

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);

    frame.render_widget(paragraph, area);
}
