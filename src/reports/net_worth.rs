//! Net Worth Report
//!
//! Generates a summary of all account balances showing total net worth.

use crate::error::EnvelopeResult;
use crate::models::{AccountId, AccountType, Money};
use crate::services::AccountService;
use crate::storage::Storage;
use std::io::Write;

/// Summary of a single account's balance
#[derive(Debug, Clone)]
pub struct AccountBalance {
    /// Account ID
    pub account_id: AccountId,
    /// Account name
    pub account_name: String,
    /// Account type
    pub account_type: AccountType,
    /// Whether this is an on-budget account
    pub on_budget: bool,
    /// Current balance
    pub balance: Money,
    /// Cleared balance
    pub cleared_balance: Money,
    /// Number of uncleared transactions
    pub uncleared_count: usize,
}

/// Net worth summary grouped by account type
#[derive(Debug, Clone)]
pub struct AccountTypeGroup {
    /// Account type
    pub account_type: AccountType,
    /// Accounts of this type
    pub accounts: Vec<AccountBalance>,
    /// Total balance for this type
    pub total_balance: Money,
    /// Total cleared balance
    pub total_cleared: Money,
}

impl AccountTypeGroup {
    /// Create a new account type group
    pub fn new(account_type: AccountType) -> Self {
        Self {
            account_type,
            accounts: Vec::new(),
            total_balance: Money::zero(),
            total_cleared: Money::zero(),
        }
    }

    /// Add an account to this group
    pub fn add_account(&mut self, account: AccountBalance) {
        self.total_balance += account.balance;
        self.total_cleared += account.cleared_balance;
        self.accounts.push(account);
    }
}

/// Net Worth Summary
#[derive(Debug, Clone)]
pub struct NetWorthSummary {
    /// Total assets (positive accounts: checking, savings, cash, investment)
    pub total_assets: Money,
    /// Total liabilities (negative accounts: credit cards, loans)
    pub total_liabilities: Money,
    /// Net worth (assets - liabilities)
    pub net_worth: Money,
    /// On-budget total
    pub on_budget_total: Money,
    /// Off-budget total
    pub off_budget_total: Money,
}

/// Net Worth Report
#[derive(Debug, Clone)]
pub struct NetWorthReport {
    /// Account groups by type
    pub groups: Vec<AccountTypeGroup>,
    /// Net worth summary
    pub summary: NetWorthSummary,
    /// Include archived accounts
    pub include_archived: bool,
}

impl NetWorthReport {
    /// Generate a net worth report
    pub fn generate(storage: &Storage, include_archived: bool) -> EnvelopeResult<Self> {
        let account_service = AccountService::new(storage);
        let summaries = account_service.list_with_balances(include_archived)?;

        // Group accounts by type
        let mut groups: std::collections::HashMap<AccountType, AccountTypeGroup> =
            std::collections::HashMap::new();

        let mut total_assets = Money::zero();
        let mut total_liabilities = Money::zero();
        let mut on_budget_total = Money::zero();
        let mut off_budget_total = Money::zero();

        for account_summary in summaries {
            let account_balance = AccountBalance {
                account_id: account_summary.account.id,
                account_name: account_summary.account.name.clone(),
                account_type: account_summary.account.account_type,
                on_budget: account_summary.account.on_budget,
                balance: account_summary.balance,
                cleared_balance: account_summary.cleared_balance,
                uncleared_count: account_summary.uncleared_count,
            };

            // Add to appropriate group
            groups
                .entry(account_summary.account.account_type)
                .or_insert_with(|| AccountTypeGroup::new(account_summary.account.account_type))
                .add_account(account_balance);

            // Track totals
            if is_liability_account(account_summary.account.account_type) {
                total_liabilities += account_summary.balance;
            } else {
                total_assets += account_summary.balance;
            }

            if account_summary.account.on_budget {
                on_budget_total += account_summary.balance;
            } else {
                off_budget_total += account_summary.balance;
            }
        }

        // Convert to sorted vector
        let mut groups: Vec<_> = groups.into_values().collect();
        groups.sort_by_key(|g| account_type_sort_order(g.account_type));

        let summary = NetWorthSummary {
            total_assets,
            total_liabilities,
            net_worth: total_assets + total_liabilities, // liabilities are already negative
            on_budget_total,
            off_budget_total,
        };

        Ok(Self {
            groups,
            summary,
            include_archived,
        })
    }

    /// Format the report for terminal display
    pub fn format_terminal(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str("Net Worth Report\n");
        output.push_str(&"=".repeat(70));
        output.push('\n');

        // Summary box
        output.push_str(&format!(
            "Total Assets:      {:>15}\n",
            self.summary.total_assets
        ));
        output.push_str(&format!(
            "Total Liabilities: {:>15}\n",
            self.summary.total_liabilities.abs()
        ));
        output.push_str(&"-".repeat(35));
        output.push('\n');
        output.push_str(&format!(
            "Net Worth:         {:>15}\n",
            self.summary.net_worth
        ));
        output.push('\n');
        output.push_str(&format!(
            "On-Budget:         {:>15}\n",
            self.summary.on_budget_total
        ));
        output.push_str(&format!(
            "Off-Budget:        {:>15}\n",
            self.summary.off_budget_total
        ));
        output.push('\n');

        // Column headers
        output.push_str(&format!(
            "{:<30} {:>12} {:>12} {:>10}\n",
            "Account", "Balance", "Cleared", "Uncleared"
        ));
        output.push_str(&"-".repeat(70));
        output.push('\n');

        // Account groups
        for group in &self.groups {
            // Group header
            output.push_str(&format!(
                "\n{}\n",
                format!("{:?}", group.account_type).to_uppercase()
            ));

            for account in &group.accounts {
                let budget_indicator = if account.on_budget { "B" } else { " " };
                output.push_str(&format!(
                    "{} {:<28} {:>12} {:>12} {:>10}\n",
                    budget_indicator,
                    account.account_name,
                    account.balance,
                    account.cleared_balance,
                    account.uncleared_count
                ));
            }

            // Group total
            output.push_str(&format!(
                "  {:<28} {:>12} {:>12}\n",
                "Subtotal:", group.total_balance, group.total_cleared
            ));
        }

        // Legend
        output.push_str(&"-".repeat(70));
        output.push('\n');
        output.push_str("B = On-Budget account\n");

        output
    }

    /// Export the report to CSV format
    pub fn export_csv<W: Write>(&self, writer: &mut W) -> EnvelopeResult<()> {
        // Write header
        writeln!(
            writer,
            "Account Type,Account Name,On Budget,Balance,Cleared Balance,Uncleared Count"
        )
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

        // Write data rows
        for group in &self.groups {
            for account in &group.accounts {
                writeln!(
                    writer,
                    "{:?},{},{},{:.2},{:.2},{}",
                    group.account_type,
                    account.account_name,
                    account.on_budget,
                    account.balance.cents() as f64 / 100.0,
                    account.cleared_balance.cents() as f64 / 100.0,
                    account.uncleared_count
                )
                .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
            }
        }

        // Summary rows
        writeln!(writer).map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
        writeln!(
            writer,
            "SUMMARY,Total Assets,,{:.2},,",
            self.summary.total_assets.cents() as f64 / 100.0
        )
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
        writeln!(
            writer,
            "SUMMARY,Total Liabilities,,{:.2},,",
            self.summary.total_liabilities.cents() as f64 / 100.0
        )
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
        writeln!(
            writer,
            "SUMMARY,Net Worth,,{:.2},,",
            self.summary.net_worth.cents() as f64 / 100.0
        )
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

        Ok(())
    }

    /// Get total number of accounts
    pub fn account_count(&self) -> usize {
        self.groups.iter().map(|g| g.accounts.len()).sum()
    }
}

/// Check if an account type is a liability
fn is_liability_account(account_type: AccountType) -> bool {
    account_type.is_liability()
}

/// Get sort order for account types (assets first, then liabilities)
fn account_type_sort_order(account_type: AccountType) -> i32 {
    match account_type {
        AccountType::Checking => 0,
        AccountType::Savings => 1,
        AccountType::Cash => 2,
        AccountType::Investment => 3,
        AccountType::Other => 4,
        AccountType::Credit => 10,
        AccountType::LineOfCredit => 11,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::EnvelopePaths;
    use crate::models::Account;
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, Storage) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        let mut storage = Storage::new(paths).unwrap();
        storage.load_all().unwrap();
        (temp_dir, storage)
    }

    #[test]
    fn test_generate_net_worth_report() {
        let (_temp_dir, storage) = create_test_storage();

        // Create accounts
        let checking = Account::with_starting_balance(
            "Checking",
            AccountType::Checking,
            Money::from_cents(500000),
        );
        storage.accounts.upsert(checking).unwrap();

        let savings = Account::with_starting_balance(
            "Savings",
            AccountType::Savings,
            Money::from_cents(1000000),
        );
        storage.accounts.upsert(savings).unwrap();

        let credit_card = Account::with_starting_balance(
            "Credit Card",
            AccountType::Credit,
            Money::from_cents(-50000),
        );
        storage.accounts.upsert(credit_card).unwrap();
        storage.accounts.save().unwrap();

        // Generate report
        let report = NetWorthReport::generate(&storage, false).unwrap();

        assert_eq!(report.account_count(), 3);
        assert_eq!(report.summary.total_assets.cents(), 1500000);
        assert_eq!(report.summary.total_liabilities.cents(), -50000);
        assert_eq!(report.summary.net_worth.cents(), 1450000);
    }

    #[test]
    fn test_csv_export() {
        let (_temp_dir, storage) = create_test_storage();

        let checking = Account::with_starting_balance(
            "Checking",
            AccountType::Checking,
            Money::from_cents(100000),
        );
        storage.accounts.upsert(checking).unwrap();
        storage.accounts.save().unwrap();

        let report = NetWorthReport::generate(&storage, false).unwrap();

        let mut csv_output = Vec::new();
        report.export_csv(&mut csv_output).unwrap();

        let csv_string = String::from_utf8(csv_output).unwrap();
        assert!(csv_string.contains("Account Type,Account Name"));
        assert!(csv_string.contains("Checking"));
        assert!(csv_string.contains("Net Worth"));
    }
}
