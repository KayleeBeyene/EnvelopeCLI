//! Unlock confirmation dialog
//!
//! Confirmation dialog for unlocking reconciled transactions.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::models::TransactionId;
use crate::tui::layout::centered_rect_fixed;

/// State for the unlock confirm dialog
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlockConfirmState {
    /// Transaction ID to unlock
    pub transaction_id: TransactionId,
    /// Transaction display info
    pub transaction_info: String,
}

impl UnlockConfirmState {
    pub fn new(transaction_id: TransactionId, info: impl Into<String>) -> Self {
        Self {
            transaction_id,
            transaction_info: info.into(),
        }
    }
}

/// Render the unlock confirmation dialog
pub fn render(frame: &mut Frame, state: &UnlockConfirmState) {
    let area = centered_rect_fixed(60, 10, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Unlock Reconciled Transaction ")
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "WARNING: This transaction has been reconciled.",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            &state.transaction_info,
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from("Editing it may cause discrepancies with your bank statement."),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Y]", Style::default().fg(Color::Yellow)),
            Span::raw(" Unlock  "),
            Span::styled("[N]", Style::default().fg(Color::Green)),
            Span::raw(" Cancel  "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
