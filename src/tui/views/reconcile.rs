//! Reconciliation view
//!
//! Displays the reconciliation workflow interface showing statement entry,
//! transaction list, and difference display.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::models::{AccountId, Money, Transaction, TransactionId, TransactionStatus};
use crate::services::ReconciliationService;
use crate::tui::app::App;

/// State for the reconciliation view
#[derive(Debug, Clone, Default)]
pub struct ReconciliationState {
    /// Account being reconciled
    pub account_id: Option<AccountId>,
    /// Statement date input
    pub statement_date: String,
    /// Statement balance input
    pub statement_balance: String,
    /// Parsed statement balance
    pub parsed_balance: Option<Money>,
    /// Whether we're in the transaction selection phase
    pub in_transaction_phase: bool,
    /// Selected transaction index
    pub selected_index: usize,
    /// List of transactions being reconciled
    pub transactions: Vec<Transaction>,
    /// Current difference
    pub difference: Money,
    /// Starting cleared balance
    pub starting_balance: Money,
    /// Active field (0=date, 1=balance)
    pub active_field: usize,
}

impl ReconciliationState {
    pub fn new() -> Self {
        Self {
            account_id: None,
            statement_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            statement_balance: String::new(),
            parsed_balance: None,
            in_transaction_phase: false,
            selected_index: 0,
            transactions: Vec::new(),
            difference: Money::zero(),
            starting_balance: Money::zero(),
            active_field: 0,
        }
    }

    /// Reset state for a new reconciliation
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Initialize for an account
    pub fn init_for_account(&mut self, account_id: AccountId) {
        self.reset();
        self.account_id = Some(account_id);
    }

    /// Get selected transaction ID
    pub fn selected_transaction(&self) -> Option<TransactionId> {
        self.transactions.get(self.selected_index).map(|t| t.id)
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.in_transaction_phase && self.selected_index > 0 {
            self.selected_index -= 1;
        } else if !self.in_transaction_phase {
            self.active_field = 0;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.in_transaction_phase {
            if self.selected_index < self.transactions.len().saturating_sub(1) {
                self.selected_index += 1;
            }
        } else {
            self.active_field = 1;
        }
    }

    /// Calculate current cleared balance
    pub fn calculate_cleared_balance(&self) -> Money {
        let cleared_total: Money = self
            .transactions
            .iter()
            .filter(|t| t.status == TransactionStatus::Cleared)
            .map(|t| t.amount)
            .sum();
        self.starting_balance + cleared_total
    }

    /// Update difference calculation
    pub fn update_difference(&mut self) {
        if let Some(statement_balance) = self.parsed_balance {
            let cleared_balance = self.calculate_cleared_balance();
            self.difference = statement_balance - cleared_balance;
        }
    }
}

/// Render the reconciliation view
pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let state = &app.reconciliation_state;

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header with statement info
            Constraint::Min(10),    // Transaction list
            Constraint::Length(4),  // Summary/status bar
        ])
        .split(area);

    render_header(frame, app, chunks[0]);
    render_transactions(frame, app, chunks[1]);
    render_summary(frame, app, chunks[2]);
}

/// Render the header with statement info
fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let state = &app.reconciliation_state;

    let account_name = if let Some(account_id) = state.account_id {
        app.storage
            .accounts
            .get(account_id)
            .ok()
            .flatten()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    } else {
        "No account selected".to_string()
    };

    let block = Block::default()
        .title(format!(" Reconcile: {} ", account_name))
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(inner);

    // Statement date
    let date_style = if !state.in_transaction_phase && state.active_field == 0 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let date_text = Paragraph::new(vec![
        Line::from(Span::styled("Statement Date:", Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled(&state.statement_date, date_style)),
    ]);
    frame.render_widget(date_text, content_chunks[0]);

    // Statement balance
    let balance_style = if !state.in_transaction_phase && state.active_field == 1 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let balance_display = if state.statement_balance.is_empty() {
        "Enter balance...".to_string()
    } else {
        state.statement_balance.clone()
    };

    let balance_text = Paragraph::new(vec![
        Line::from(Span::styled("Statement Balance:", Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled(balance_display, balance_style)),
    ]);
    frame.render_widget(balance_text, content_chunks[1]);
}

/// Render the transaction list
fn render_transactions(frame: &mut Frame, app: &App, area: Rect) {
    let state = &app.reconciliation_state;

    let block = Block::default()
        .title(" Transactions ")
        .title_style(Style::default().fg(Color::White))
        .borders(Borders::ALL)
        .border_style(if state.in_transaction_phase {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    if state.transactions.is_empty() {
        let empty_msg = Paragraph::new("No transactions to reconcile")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_msg, area);
        return;
    }

    let header = Row::new(vec!["", "Date", "Payee", "Amount", "Status"])
        .style(Style::default().fg(Color::DarkGray))
        .bottom_margin(1);

    let rows: Vec<Row> = state
        .transactions
        .iter()
        .enumerate()
        .map(|(i, txn)| {
            let is_selected = state.in_transaction_phase && i == state.selected_index;

            let status_icon = match txn.status {
                TransactionStatus::Pending => "[ ]",
                TransactionStatus::Cleared => "[C]",
                TransactionStatus::Reconciled => "[R]",
            };

            let status_color = match txn.status {
                TransactionStatus::Pending => Color::DarkGray,
                TransactionStatus::Cleared => Color::Green,
                TransactionStatus::Reconciled => Color::Blue,
            };

            let row_style = if is_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            Row::new(vec![
                status_icon.to_string(),
                txn.date.to_string(),
                txn.payee_name.chars().take(30).collect::<String>(),
                format!("{:>12}", txn.amount),
                txn.status.to_string(),
            ])
            .style(row_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(12),
            Constraint::Min(20),
            Constraint::Length(14),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(block);

    frame.render_widget(table, area);
}

/// Render the summary bar
fn render_summary(frame: &mut Frame, app: &App, area: Rect) {
    let state = &app.reconciliation_state;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cleared_balance = state.calculate_cleared_balance();

    let difference_style = if state.difference.is_zero() {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    };

    let cleared_count = state
        .transactions
        .iter()
        .filter(|t| t.status == TransactionStatus::Cleared)
        .count();

    let pending_count = state
        .transactions
        .iter()
        .filter(|t| t.status == TransactionStatus::Pending)
        .count();

    let summary_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(inner);

    // Cleared balance
    let cleared_text = Paragraph::new(vec![Line::from(vec![
        Span::styled("Cleared: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", cleared_balance), Style::default().fg(Color::White)),
    ])]);
    frame.render_widget(cleared_text, summary_chunks[0]);

    // Difference
    let diff_text = Paragraph::new(vec![Line::from(vec![
        Span::styled("Difference: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", state.difference), difference_style),
    ])]);
    frame.render_widget(diff_text, summary_chunks[1]);

    // Transaction counts
    let count_text = Paragraph::new(vec![Line::from(vec![
        Span::styled(format!("{} cleared  ", cleared_count), Style::default().fg(Color::Green)),
        Span::styled(format!("{} pending", pending_count), Style::default().fg(Color::DarkGray)),
    ])]);
    frame.render_widget(count_text, summary_chunks[2]);
}

/// Handle key input for reconciliation view
pub fn handle_key(app: &mut App, key: crossterm::event::KeyCode) -> bool {
    use crossterm::event::KeyCode;

    let state = &mut app.reconciliation_state;

    match key {
        KeyCode::Up | KeyCode::Char('k') => {
            state.move_up();
            true
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.move_down();
            true
        }
        KeyCode::Tab => {
            // Toggle between header and transaction phases
            if !state.in_transaction_phase && state.parsed_balance.is_some() {
                state.in_transaction_phase = true;
            } else {
                state.in_transaction_phase = false;
            }
            true
        }
        KeyCode::Char(' ') if state.in_transaction_phase => {
            // Toggle cleared status on selected transaction
            if let Some(txn_id) = state.selected_transaction() {
                let service = ReconciliationService::new(app.storage);
                if let Some(txn) = state.transactions.iter().find(|t| t.id == txn_id) {
                    let result = if txn.status == TransactionStatus::Cleared {
                        service.unclear_transaction(txn_id)
                    } else if txn.status == TransactionStatus::Pending {
                        service.clear_transaction(txn_id)
                    } else {
                        // Reconciled - can't change
                        return true;
                    };

                    if let Ok(updated_txn) = result {
                        // Update the transaction in our list
                        if let Some(t) = state.transactions.iter_mut().find(|t| t.id == txn_id) {
                            *t = updated_txn;
                        }
                        state.update_difference();
                    }
                }
            }
            true
        }
        KeyCode::Enter if !state.in_transaction_phase => {
            // Move to transaction phase
            if state.active_field == 1 {
                // Try to parse the balance
                if let Ok(balance) = Money::parse(&state.statement_balance) {
                    state.parsed_balance = Some(balance);
                    state.in_transaction_phase = true;

                    // Load transactions
                    if let Some(account_id) = state.account_id {
                        let service = ReconciliationService::new(app.storage);
                        if let Ok(transactions) = service.get_uncleared_transactions(account_id) {
                            state.transactions = transactions;
                        }

                        // Calculate starting balance
                        if let Ok(Some(account)) = app.storage.accounts.get(account_id) {
                            let reconciled_total: Money = app
                                .storage
                                .transactions
                                .get_by_account(account_id)
                                .unwrap_or_default()
                                .iter()
                                .filter(|t| t.status == TransactionStatus::Reconciled)
                                .map(|t| t.amount)
                                .sum();
                            state.starting_balance = account.starting_balance + reconciled_total;
                        }

                        state.update_difference();
                    }
                } else {
                    app.set_status("Invalid balance format. Use format like 1234.56");
                }
            } else {
                state.active_field = 1;
            }
            true
        }
        KeyCode::Char(c) if !state.in_transaction_phase => {
            // Input to the active field
            if state.active_field == 0 {
                state.statement_date.push(c);
            } else {
                state.statement_balance.push(c);
            }
            true
        }
        KeyCode::Backspace if !state.in_transaction_phase => {
            if state.active_field == 0 {
                state.statement_date.pop();
            } else {
                state.statement_balance.pop();
            }
            true
        }
        _ => false,
    }
}
