//! Transaction display formatting
//!
//! Provides utilities for formatting transactions for terminal display,
//! including register views and status indicators.

use crate::models::{Transaction, TransactionStatus};

/// Format a single transaction for display (register row)
pub fn format_transaction_row(txn: &Transaction) -> String {
    let status_icon = match txn.status {
        TransactionStatus::Pending => " ",
        TransactionStatus::Cleared => "✓",
        TransactionStatus::Reconciled => "🔒",
    };

    let transfer_indicator = if txn.is_transfer() { "⇄ " } else { "" };
    let split_indicator = if txn.is_split() {
        format!(" [{}]", txn.splits.len())
    } else {
        String::new()
    };

    let payee_display = if txn.payee_name.is_empty() {
        "(no payee)".to_string()
    } else {
        format!("{}{}", transfer_indicator, txn.payee_name)
    };

    format!(
        "{} {} {:20} {:>12}{}",
        status_icon,
        txn.date.format("%Y-%m-%d"),
        truncate(&payee_display, 20),
        txn.amount,
        split_indicator
    )
}

/// Format a list of transactions as a register
pub fn format_transaction_register(transactions: &[Transaction]) -> String {
    if transactions.is_empty() {
        return "No transactions found.\n".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!(
        "{:3} {:10} {:20} {:>12}\n",
        "St", "Date", "Payee", "Amount"
    ));
    output.push_str(&"-".repeat(50));
    output.push('\n');

    for txn in transactions {
        output.push_str(&format_transaction_row(txn));
        output.push('\n');
    }

    output
}

/// Format transaction details for display
pub fn format_transaction_details(txn: &Transaction, category_name: Option<&str>) -> String {
    let mut output = String::new();

    output.push_str(&format!("Transaction: {}\n", txn.id));
    output.push_str(&format!("Date:        {}\n", txn.date.format("%Y-%m-%d")));
    output.push_str(&format!("Amount:      {}\n", txn.amount));

    if !txn.payee_name.is_empty() {
        output.push_str(&format!("Payee:       {}\n", txn.payee_name));
    }

    if let Some(cat_name) = category_name {
        output.push_str(&format!("Category:    {}\n", cat_name));
    } else if txn.is_split() {
        output.push_str(&format!("Category:    Split ({} categories)\n", txn.splits.len()));
    } else {
        output.push_str("Category:    (uncategorized)\n");
    }

    if !txn.memo.is_empty() {
        output.push_str(&format!("Memo:        {}\n", txn.memo));
    }

    output.push_str(&format!("Status:      {}\n", txn.status));

    if txn.is_transfer() {
        output.push_str("Type:        Transfer\n");
    }

    if txn.is_split() {
        output.push_str("\nSplits:\n");
        for (i, split) in txn.splits.iter().enumerate() {
            let memo_part = if split.memo.is_empty() {
                String::new()
            } else {
                format!(" - {}", split.memo)
            };
            output.push_str(&format!(
                "  {}. {} to {}{}\n",
                i + 1,
                split.amount,
                split.category_id,
                memo_part
            ));
        }
    }

    output
}

/// Format a transaction list with account grouping
pub fn format_transaction_list_by_account(
    transactions: &[Transaction],
    account_name: &str,
) -> String {
    let mut output = String::new();

    output.push_str(&format!("Account: {}\n", account_name));
    output.push_str(&format!("Transactions: {}\n\n", transactions.len()));

    output.push_str(&format!(
        "{:3} {:10} {:20} {:>12} {:>12}\n",
        "St", "Date", "Payee", "Outflow", "Inflow"
    ));
    output.push_str(&"-".repeat(62));
    output.push('\n');

    let mut running_balance = crate::models::Money::zero();

    for txn in transactions {
        let status_icon = match txn.status {
            TransactionStatus::Pending => " ",
            TransactionStatus::Cleared => "✓",
            TransactionStatus::Reconciled => "🔒",
        };

        let payee_display = if txn.payee_name.is_empty() {
            "(no payee)".to_string()
        } else {
            txn.payee_name.clone()
        };

        let (outflow, inflow) = if txn.amount.is_negative() {
            (format!("{}", -txn.amount), String::new())
        } else {
            (String::new(), format!("{}", txn.amount))
        };

        running_balance += txn.amount;

        output.push_str(&format!(
            "{:3} {} {:20} {:>12} {:>12}\n",
            status_icon,
            txn.date.format("%Y-%m-%d"),
            truncate(&payee_display, 20),
            outflow,
            inflow
        ));
    }

    output.push_str(&"-".repeat(62));
    output.push('\n');
    output.push_str(&format!("{:>50} {:>12}\n", "Balance:", running_balance));

    output
}

/// Format a short transaction summary (one line)
pub fn format_transaction_short(txn: &Transaction) -> String {
    let status_icon = match txn.status {
        TransactionStatus::Pending => " ",
        TransactionStatus::Cleared => "✓",
        TransactionStatus::Reconciled => "🔒",
    };

    let payee_display = if txn.payee_name.is_empty() {
        "(no payee)"
    } else {
        &txn.payee_name
    };

    format!(
        "{} {} {} {}",
        status_icon,
        txn.date.format("%Y-%m-%d"),
        truncate(payee_display, 20),
        txn.amount
    )
}

/// Truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AccountId, Money};
    use chrono::NaiveDate;

    #[test]
    fn test_format_transaction_row() {
        let txn = Transaction::with_details(
            AccountId::new(),
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            Money::from_cents(-5000),
            "Test Store",
            None,
            "",
        );

        let formatted = format_transaction_row(&txn);
        assert!(formatted.contains("2025-01-15"));
        assert!(formatted.contains("Test Store"));
        assert!(formatted.contains("-$50.00"));
    }

    #[test]
    fn test_format_empty_register() {
        let formatted = format_transaction_register(&[]);
        assert!(formatted.contains("No transactions found"));
    }

    #[test]
    fn test_format_transaction_details() {
        let txn = Transaction::with_details(
            AccountId::new(),
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            Money::from_cents(-5000),
            "Test Store",
            None,
            "Test memo",
        );

        let formatted = format_transaction_details(&txn, Some("Groceries"));
        assert!(formatted.contains("Test Store"));
        assert!(formatted.contains("Groceries"));
        assert!(formatted.contains("Test memo"));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Short", 10).trim(), "Short");
        // Note: truncate pads short strings, so we test the truncation behavior
        let result = truncate("A very long string", 10);
        assert!(result.len() <= 10);
        assert!(result.ends_with("..."));
    }
}
