//! Transaction register view
//!
//! Shows transactions for the selected account

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::models::TransactionStatus;
use crate::tui::app::{App, FocusedPanel};
use crate::tui::layout::MainPanelLayout;

/// Render the transaction register
pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let layout = MainPanelLayout::new(area);

    // Render header with account name
    render_header(frame, app, layout.header);

    // Render transaction table
    render_transaction_table(frame, app, layout.content);
}

/// Render register header
fn render_header(frame: &mut Frame, app: &mut App, area: Rect) {
    let account_name = if let Some(account_id) = app.selected_account {
        app.storage
            .accounts
            .get(account_id)
            .ok()
            .flatten()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    } else {
        "No Account Selected".to_string()
    };

    let title = format!(" {} - Transactions ", account_name);
    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let hints = if app.multi_select_mode {
        "Multi-select: SPACE to select, C to categorize, v to exit"
    } else {
        "a:Add  e:Edit  c:Clear  v:Multi-select"
    };

    let paragraph = Paragraph::new(hints)
        .block(block)
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}

/// Render transaction table
fn render_transaction_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Main;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    // Get transactions for selected account
    let transactions = if let Some(account_id) = app.selected_account {
        let mut txns = app
            .storage
            .transactions
            .get_by_account(account_id)
            .unwrap_or_default();
        // Sort by date descending
        txns.sort_by(|a, b| b.date.cmp(&a.date));
        txns
    } else {
        Vec::new()
    };

    if transactions.is_empty() {
        let text = Paragraph::new("No transactions. Press 'a' to add one.")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, area);
        return;
    }

    // Define column widths
    let widths = [
        ratatui::layout::Constraint::Length(2),  // Status
        ratatui::layout::Constraint::Length(12), // Date
        ratatui::layout::Constraint::Length(20), // Payee
        ratatui::layout::Constraint::Length(15), // Category
        ratatui::layout::Constraint::Length(12), // Amount
        ratatui::layout::Constraint::Min(10),    // Memo
    ];

    // Header row
    let header = Row::new(vec![
        Cell::from(""),
        Cell::from("Date").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Payee").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Category").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Amount").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Memo").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().fg(Color::Yellow))
    .height(1);

    // Get categories for lookup
    let categories = app.storage.categories.get_all_categories().unwrap_or_default();

    // Data rows
    let rows: Vec<Row> = transactions
        .iter()
        .map(|txn| {
            // Status indicator
            let status_indicator = match txn.status {
                TransactionStatus::Pending => "○",
                TransactionStatus::Cleared => "✓",
                TransactionStatus::Reconciled => "🔒",
            };
            let status_color = match txn.status {
                TransactionStatus::Pending => Color::Yellow,
                TransactionStatus::Cleared => Color::Green,
                TransactionStatus::Reconciled => Color::Blue,
            };

            // Multi-select indicator
            let is_selected = app.selected_transactions.contains(&txn.id);
            let select_indicator = if app.multi_select_mode {
                if is_selected { "■ " } else { "□ " }
            } else {
                ""
            };

            // Category name
            let category_name = if txn.is_split() {
                "Split".to_string()
            } else if txn.is_transfer() {
                "Transfer".to_string()
            } else if let Some(cat_id) = txn.category_id {
                categories
                    .iter()
                    .find(|c| c.id == cat_id)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "-".to_string()
            };

            // Amount styling
            let amount_style = if txn.amount.is_negative() {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            };

            Row::new(vec![
                Cell::from(format!("{}{}", select_indicator, status_indicator))
                    .style(Style::default().fg(status_color)),
                Cell::from(txn.date.format("%Y-%m-%d").to_string()),
                Cell::from(truncate_string(&txn.payee_name, 20)),
                Cell::from(truncate_string(&category_name, 15)),
                Cell::from(format!("{}", txn.amount)).style(amount_style),
                Cell::from(truncate_string(&txn.memo, 30)),
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
    state.select(Some(app.selected_transaction_index));

    frame.render_stateful_widget(table, area, &mut state);
}

/// Truncate a string to a maximum length
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
