//! Account display formatting
//!
//! Formats accounts for terminal output in table and detail views.

use crate::models::Account;
use crate::services::account::AccountSummary;
use tabled::{
    settings::{object::Columns, Alignment, Modify, Style},
    Table, Tabled,
};

/// Row for account table display (used in pretty mode)
#[derive(Tabled)]
struct AccountRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    account_type: String,
    #[tabled(rename = "Balance")]
    balance: String,
    #[tabled(rename = "Cleared")]
    cleared: String,
    #[tabled(rename = "Status")]
    status: String,
}

/// Format a list of accounts with balances as a table
///
/// When `pretty` is true, uses a bordered table with rounded corners.
/// When `pretty` is false, uses a simple text-based table format.
pub fn format_account_list(summaries: &[AccountSummary], pretty: bool) -> String {
    if summaries.is_empty() {
        return "No accounts found.".to_string();
    }

    if pretty {
        format_account_list_pretty(summaries)
    } else {
        format_account_list_plain(summaries)
    }
}

/// Format account list with bordered table (pretty mode)
fn format_account_list_pretty(summaries: &[AccountSummary]) -> String {
    let mut rows: Vec<AccountRow> = summaries
        .iter()
        .map(|summary| {
            let status = get_account_status(summary);
            AccountRow {
                name: summary.account.name.clone(),
                account_type: summary.account.account_type.to_string(),
                balance: summary.balance.to_string(),
                cleared: summary.cleared_balance.to_string(),
                status,
            }
        })
        .collect();

    // Add total row
    let total_balance: crate::models::Money = summaries.iter().map(|s| s.balance).sum();
    let total_cleared: crate::models::Money = summaries.iter().map(|s| s.cleared_balance).sum();

    rows.push(AccountRow {
        name: "TOTAL".to_string(),
        account_type: String::new(),
        balance: total_balance.to_string(),
        cleared: total_cleared.to_string(),
        status: String::new(),
    });

    Table::new(&rows)
        .with(Style::rounded())
        .with(Modify::new(Columns::new(2..=3)).with(Alignment::right()))
        .to_string()
}

/// Format account list with simple text table (plain mode)
/// Matches the budget overview formatting style (80-char width)
fn format_account_list_plain(summaries: &[AccountSummary]) -> String {
    let mut output = String::new();

    // Header - matches budget overview style
    output.push_str("Accounts\n");
    output.push_str(&"=".repeat(80));
    output.push('\n');

    // Column headers - aligned with data rows
    // Widths: Name=26, Type=14, Balance=12, Cleared=12, Status=12 (+ 4 spaces = 80)
    output.push_str(&format!(
        "{:<26} {:>14} {:>12} {:>12} {:>12}\n",
        "Name", "Type", "Balance", "Cleared", "Status"
    ));
    output.push_str(&"-".repeat(80));
    output.push('\n');

    // Account rows - same column widths as header
    // Note: account_type.to_string() is needed because AccountType's Display
    // impl doesn't honor width specifiers, unlike Money which does
    for summary in summaries {
        let status = get_account_status(summary);
        output.push_str(&format!(
            "{:<26} {:>14} {:>12} {:>12} {:>12}\n",
            truncate_str(&summary.account.name, 26),
            summary.account.account_type.to_string(),
            summary.balance,
            summary.cleared_balance,
            status,
        ));
    }

    // Total row
    let total_balance: crate::models::Money = summaries.iter().map(|s| s.balance).sum();
    let total_cleared: crate::models::Money = summaries.iter().map(|s| s.cleared_balance).sum();

    output.push('\n');
    output.push_str(&"=".repeat(80));
    output.push('\n');
    output.push_str(&format!(
        "{:<26} {:>14} {:>12} {:>12}\n",
        "TOTALS:", "", total_balance, total_cleared
    ));

    output
}

/// Truncate a string to a maximum length, adding "..." if truncated
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}

/// Get status string for an account
fn get_account_status(summary: &AccountSummary) -> String {
    if summary.account.archived {
        "Archived".to_string()
    } else if !summary.account.on_budget {
        "Off-Budget".to_string()
    } else if summary.uncleared_count > 0 {
        format!("{} pending", summary.uncleared_count)
    } else {
        "Active".to_string()
    }
}

/// Format a single account's details
pub fn format_account_details(summary: &AccountSummary) -> String {
    let account = &summary.account;

    let mut output = String::new();

    output.push_str(&format!("Account: {}\n", account.name));
    output.push_str(&format!("  Type:           {}\n", account.account_type));
    output.push_str(&format!("  ID:             {}\n", account.id));
    output.push_str(&format!(
        "  On Budget:      {}\n",
        if account.on_budget { "Yes" } else { "No" }
    ));
    output.push_str(&format!(
        "  Archived:       {}\n",
        if account.archived { "Yes" } else { "No" }
    ));
    output.push('\n');
    output.push_str(&format!(
        "  Starting Balance: {}\n",
        account.starting_balance
    ));
    output.push_str(&format!("  Current Balance:  {}\n", summary.balance));
    output.push_str(&format!(
        "  Cleared Balance:  {}\n",
        summary.cleared_balance
    ));
    output.push_str(&format!(
        "  Uncleared Count:  {}\n",
        summary.uncleared_count
    ));

    if let Some(date) = account.last_reconciled_date {
        output.push('\n');
        output.push_str(&format!("  Last Reconciled:  {}\n", date));
        if let Some(balance) = account.last_reconciled_balance {
            output.push_str(&format!("  Reconciled Balance: {}\n", balance));
        }
    }

    if !account.notes.is_empty() {
        output.push('\n');
        output.push_str(&format!("  Notes: {}\n", account.notes));
    }

    output.push('\n');
    output.push_str(&format!(
        "  Created:  {}\n",
        account.created_at.format("%Y-%m-%d %H:%M UTC")
    ));
    output.push_str(&format!(
        "  Modified: {}\n",
        account.updated_at.format("%Y-%m-%d %H:%M UTC")
    ));

    output
}

/// Format a simple account list (name and type only)
pub fn format_account_list_simple(accounts: &[Account]) -> String {
    if accounts.is_empty() {
        return "No accounts found.".to_string();
    }

    let mut output = String::new();
    for account in accounts {
        let status = if account.archived { " (archived)" } else { "" };
        output.push_str(&format!(
            "  {} - {}{}\n",
            account.name, account.account_type, status
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AccountType, Money};

    fn create_test_summary(name: &str, balance: i64, cleared: i64) -> AccountSummary {
        let account =
            Account::with_starting_balance(name, AccountType::Checking, Money::from_cents(0));
        AccountSummary {
            account,
            balance: Money::from_cents(balance),
            cleared_balance: Money::from_cents(cleared),
            uncleared_count: if balance != cleared { 1 } else { 0 },
        }
    }

    #[test]
    fn test_format_account_list_plain() {
        let summaries = vec![
            create_test_summary("Checking", 100000, 95000),
            create_test_summary("Savings", 500000, 500000),
        ];

        let output = format_account_list(&summaries, false);
        assert!(output.contains("Accounts"));
        assert!(output.contains("===="));  // 80-char header separator
        assert!(output.contains("----"));  // 80-char row separator
        assert!(output.contains("Checking"));
        assert!(output.contains("Savings"));
        assert!(output.contains("TOTALS:")); // Matches budget overview style
    }

    #[test]
    fn test_format_account_list_pretty() {
        let summaries = vec![
            create_test_summary("Checking", 100000, 95000),
            create_test_summary("Savings", 500000, 500000),
        ];

        let output = format_account_list(&summaries, true);
        assert!(output.contains("Checking"));
        assert!(output.contains("Savings"));
        assert!(output.contains("TOTAL"));
        assert!(output.contains("│")); // Pretty mode has box drawing chars
    }

    #[test]
    fn test_format_empty_list() {
        let output = format_account_list(&[], false);
        assert!(output.contains("No accounts found"));
    }

    #[test]
    fn test_format_account_details() {
        let summary = create_test_summary("My Account", 100000, 90000);
        let output = format_account_details(&summary);

        assert!(output.contains("My Account"));
        assert!(output.contains("Checking"));
        assert!(output.contains("Current Balance"));
        assert!(output.contains("Cleared Balance"));
    }
}
