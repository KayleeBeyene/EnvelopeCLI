//! Account display formatting
//!
//! Formats accounts for terminal output in table and detail views.

use crate::models::Account;
use crate::services::account::AccountSummary;

/// Format a list of accounts with balances as a table
pub fn format_account_list(summaries: &[AccountSummary]) -> String {
    if summaries.is_empty() {
        return "No accounts found.".to_string();
    }

    // Calculate column widths
    let name_width = summaries
        .iter()
        .map(|s| s.account.name.len())
        .max()
        .unwrap_or(4)
        .max(4);

    let type_width = summaries
        .iter()
        .map(|s| s.account.account_type.to_string().len())
        .max()
        .unwrap_or(4)
        .max(4);

    // Build header
    let mut output = String::new();
    output.push_str(&format!(
        "{:<name_width$}  {:<type_width$}  {:>12}  {:>12}  {}\n",
        "Name",
        "Type",
        "Balance",
        "Cleared",
        "Status",
        name_width = name_width,
        type_width = type_width,
    ));

    // Separator line
    output.push_str(&format!(
        "{:-<name_width$}  {:-<type_width$}  {:->12}  {:->12}  {:-<10}\n",
        "",
        "",
        "",
        "",
        "",
        name_width = name_width,
        type_width = type_width,
    ));

    // Account rows
    for summary in summaries {
        let status = if summary.account.archived {
            "Archived"
        } else if !summary.account.on_budget {
            "Off-Budget"
        } else if summary.uncleared_count > 0 {
            &format!("{} pending", summary.uncleared_count)
        } else {
            ""
        };

        output.push_str(&format!(
            "{:<name_width$}  {:<type_width$}  {:>12}  {:>12}  {}\n",
            summary.account.name,
            summary.account.account_type,
            summary.balance.to_string(),
            summary.cleared_balance.to_string(),
            status,
            name_width = name_width,
            type_width = type_width,
        ));
    }

    // Total row
    let total_balance: crate::models::Money = summaries.iter().map(|s| s.balance).sum();
    let total_cleared: crate::models::Money = summaries.iter().map(|s| s.cleared_balance).sum();

    output.push_str(&format!(
        "{:-<name_width$}  {:-<type_width$}  {:->12}  {:->12}  {:-<10}\n",
        "",
        "",
        "",
        "",
        "",
        name_width = name_width,
        type_width = type_width,
    ));

    output.push_str(&format!(
        "{:<name_width$}  {:<type_width$}  {:>12}  {:>12}\n",
        "TOTAL",
        "",
        total_balance.to_string(),
        total_cleared.to_string(),
        name_width = name_width,
        type_width = type_width,
    ));

    output
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
    fn test_format_account_list() {
        let summaries = vec![
            create_test_summary("Checking", 100000, 95000),
            create_test_summary("Savings", 500000, 500000),
        ];

        let output = format_account_list(&summaries);
        assert!(output.contains("Checking"));
        assert!(output.contains("Savings"));
        assert!(output.contains("TOTAL"));
        assert!(
            output.contains("$1,000.00")
                || output.contains("$6000.00")
                || output.contains("$6,000.00")
        );
    }

    #[test]
    fn test_format_empty_list() {
        let output = format_account_list(&[]);
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
