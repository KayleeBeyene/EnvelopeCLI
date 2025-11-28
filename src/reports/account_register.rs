//! Account Register Report
//!
//! Generates a detailed transaction register for an account with filtering options.

use crate::error::EnvelopeResult;
use crate::models::{AccountId, CategoryId, Money, Transaction, TransactionStatus};
use crate::services::{AccountService, CategoryService};
use crate::storage::Storage;
use chrono::NaiveDate;
use std::io::Write;

/// A single entry in the register report
#[derive(Debug, Clone)]
pub struct RegisterEntry {
    /// Transaction date
    pub date: NaiveDate,
    /// Payee name
    pub payee: String,
    /// Category name (or "Split" or "Transfer" or "Uncategorized")
    pub category: String,
    /// Memo
    pub memo: String,
    /// Transaction amount
    pub amount: Money,
    /// Running balance after this transaction
    pub running_balance: Money,
    /// Transaction status
    pub status: TransactionStatus,
    /// Whether this is a split transaction
    pub is_split: bool,
    /// Whether this is a transfer
    pub is_transfer: bool,
}

/// Filter options for the register report
#[derive(Debug, Clone, Default)]
pub struct RegisterFilter {
    /// Filter by start date
    pub start_date: Option<NaiveDate>,
    /// Filter by end date
    pub end_date: Option<NaiveDate>,
    /// Filter by category ID
    pub category_id: Option<CategoryId>,
    /// Filter by status
    pub status: Option<TransactionStatus>,
    /// Filter by payee (partial match)
    pub payee_contains: Option<String>,
    /// Filter by minimum amount (absolute value)
    pub min_amount: Option<Money>,
    /// Filter by maximum amount (absolute value)
    pub max_amount: Option<Money>,
    /// Only show uncategorized transactions
    pub uncategorized_only: bool,
}

impl RegisterFilter {
    /// Check if a transaction matches this filter
    pub fn matches(&self, txn: &Transaction) -> bool {
        // Date filters
        if let Some(start) = self.start_date {
            if txn.date < start {
                return false;
            }
        }
        if let Some(end) = self.end_date {
            if txn.date > end {
                return false;
            }
        }

        // Category filter
        if let Some(cat_id) = self.category_id {
            let matches_category = txn.category_id == Some(cat_id)
                || txn.splits.iter().any(|s| s.category_id == cat_id);
            if !matches_category {
                return false;
            }
        }

        // Status filter
        if let Some(status) = self.status {
            if txn.status != status {
                return false;
            }
        }

        // Payee filter
        if let Some(ref payee) = self.payee_contains {
            if !txn.payee_name.to_lowercase().contains(&payee.to_lowercase()) {
                return false;
            }
        }

        // Amount filters
        let abs_amount = txn.amount.abs();
        if let Some(min) = self.min_amount {
            if abs_amount < min {
                return false;
            }
        }
        if let Some(max) = self.max_amount {
            if abs_amount > max {
                return false;
            }
        }

        // Uncategorized filter
        if self.uncategorized_only {
            if txn.category_id.is_some() || !txn.splits.is_empty() || txn.is_transfer() {
                return false;
            }
        }

        true
    }
}

/// Account Register Report
#[derive(Debug, Clone)]
pub struct AccountRegisterReport {
    /// Account ID
    pub account_id: AccountId,
    /// Account name
    pub account_name: String,
    /// Starting balance (before first transaction in report)
    pub starting_balance: Money,
    /// Ending balance (after last transaction)
    pub ending_balance: Money,
    /// Register entries
    pub entries: Vec<RegisterEntry>,
    /// Total inflows in the report period
    pub total_inflows: Money,
    /// Total outflows in the report period
    pub total_outflows: Money,
    /// Filter applied
    pub filter: RegisterFilter,
}

impl AccountRegisterReport {
    /// Generate a register report for an account
    pub fn generate(
        storage: &Storage,
        account_id: AccountId,
        filter: RegisterFilter,
    ) -> EnvelopeResult<Self> {
        let account_service = AccountService::new(storage);
        let category_service = CategoryService::new(storage);

        // Get the account
        let account = account_service
            .get(account_id)?
            .ok_or_else(|| crate::error::EnvelopeError::account_not_found(account_id.to_string()))?;

        // Build category lookup
        let categories = category_service.list_categories()?;
        let category_names: std::collections::HashMap<CategoryId, String> = categories
            .iter()
            .map(|c| (c.id, c.name.clone()))
            .collect();

        // Get all transactions for this account
        let mut transactions = storage.transactions.get_by_account(account_id)?;

        // Sort by date, then by created_at for same-day transactions
        transactions.sort_by(|a, b| {
            a.date
                .cmp(&b.date)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        // Calculate starting balance (account starting balance + all transactions before filter start)
        let mut starting_balance = account.starting_balance;
        if let Some(start_date) = filter.start_date {
            for txn in &transactions {
                if txn.date < start_date {
                    starting_balance += txn.amount;
                }
            }
        }

        // Build register entries
        let mut entries = Vec::new();
        let mut running_balance = starting_balance;
        let mut total_inflows = Money::zero();
        let mut total_outflows = Money::zero();

        for txn in &transactions {
            // Apply filter
            if !filter.matches(txn) {
                continue;
            }

            // Update running balance
            running_balance += txn.amount;

            // Track totals
            if txn.amount.is_positive() {
                total_inflows += txn.amount;
            } else {
                total_outflows += txn.amount;
            }

            // Determine category display
            let category = if txn.is_transfer() {
                "Transfer".to_string()
            } else if txn.is_split() {
                "Split".to_string()
            } else if let Some(cat_id) = txn.category_id {
                category_names.get(&cat_id).cloned().unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Uncategorized".to_string()
            };

            entries.push(RegisterEntry {
                date: txn.date,
                payee: txn.payee_name.clone(),
                category,
                memo: txn.memo.clone(),
                amount: txn.amount,
                running_balance,
                status: txn.status,
                is_split: txn.is_split(),
                is_transfer: txn.is_transfer(),
            });
        }

        Ok(Self {
            account_id,
            account_name: account.name.clone(),
            starting_balance,
            ending_balance: running_balance,
            entries,
            total_inflows,
            total_outflows,
            filter,
        })
    }

    /// Format the report for terminal display
    pub fn format_terminal(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("Account Register: {}\n", self.account_name));
        output.push_str(&"=".repeat(100));
        output.push('\n');

        // Filter info
        if let Some(start) = self.filter.start_date {
            output.push_str(&format!("From: {} ", start));
        }
        if let Some(end) = self.filter.end_date {
            output.push_str(&format!("To: {} ", end));
        }
        output.push('\n');

        output.push_str(&format!("Starting Balance: {}\n", self.starting_balance));
        output.push_str(&format!("Ending Balance:   {}\n\n", self.ending_balance));

        // Column headers
        output.push_str(&format!(
            "{:<12} {:<20} {:<20} {:>12} {:>12} {:>4}\n",
            "Date", "Payee", "Category", "Amount", "Balance", "Clr"
        ));
        output.push_str(&"-".repeat(100));
        output.push('\n');

        // Entries
        for entry in &self.entries {
            let status_char = match entry.status {
                TransactionStatus::Pending => " ",
                TransactionStatus::Cleared => "C",
                TransactionStatus::Reconciled => "R",
            };

            let payee_display = if entry.payee.len() > 18 {
                format!("{}...", &entry.payee[..15])
            } else {
                entry.payee.clone()
            };

            let category_display = if entry.category.len() > 18 {
                format!("{}...", &entry.category[..15])
            } else {
                entry.category.clone()
            };

            output.push_str(&format!(
                "{:<12} {:<20} {:<20} {:>12} {:>12} {:>4}\n",
                entry.date,
                payee_display,
                category_display,
                entry.amount,
                entry.running_balance,
                status_char
            ));
        }

        // Summary
        output.push_str(&"-".repeat(100));
        output.push('\n');
        output.push_str(&format!(
            "Total Inflows:  {}  |  Total Outflows: {}  |  Transactions: {}\n",
            self.total_inflows,
            self.total_outflows.abs(),
            self.entries.len()
        ));

        output
    }

    /// Export the report to CSV format
    pub fn export_csv<W: Write>(&self, writer: &mut W) -> EnvelopeResult<()> {
        // Write header
        writeln!(
            writer,
            "Account,Date,Payee,Category,Memo,Amount,Running Balance,Status"
        )
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

        // Write data rows
        for entry in &self.entries {
            let status = match entry.status {
                TransactionStatus::Pending => "Pending",
                TransactionStatus::Cleared => "Cleared",
                TransactionStatus::Reconciled => "Reconciled",
            };

            // Escape CSV fields that might contain commas
            let payee = escape_csv_field(&entry.payee);
            let category = escape_csv_field(&entry.category);
            let memo = escape_csv_field(&entry.memo);

            writeln!(
                writer,
                "{},{},{},{},{},{:.2},{:.2},{}",
                self.account_name,
                entry.date,
                payee,
                category,
                memo,
                entry.amount.cents() as f64 / 100.0,
                entry.running_balance.cents() as f64 / 100.0,
                status
            )
            .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
        }

        Ok(())
    }

    /// Get summary statistics
    pub fn summary(&self) -> RegisterSummary {
        let cleared_count = self
            .entries
            .iter()
            .filter(|e| matches!(e.status, TransactionStatus::Cleared | TransactionStatus::Reconciled))
            .count();

        let pending_count = self
            .entries
            .iter()
            .filter(|e| e.status == TransactionStatus::Pending)
            .count();

        RegisterSummary {
            total_entries: self.entries.len(),
            cleared_count,
            pending_count,
            total_inflows: self.total_inflows,
            total_outflows: self.total_outflows,
            net_change: self.total_inflows + self.total_outflows,
        }
    }
}

/// Summary statistics for a register report
#[derive(Debug, Clone)]
pub struct RegisterSummary {
    /// Total number of entries
    pub total_entries: usize,
    /// Number of cleared/reconciled entries
    pub cleared_count: usize,
    /// Number of pending entries
    pub pending_count: usize,
    /// Total inflows
    pub total_inflows: Money,
    /// Total outflows
    pub total_outflows: Money,
    /// Net change (inflows + outflows)
    pub net_change: Money,
}

/// Escape a string for CSV format
fn escape_csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::EnvelopePaths;
    use crate::models::{Account, AccountType, Category, CategoryGroup};
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, Storage) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        let mut storage = Storage::new(paths).unwrap();
        storage.load_all().unwrap();
        (temp_dir, storage)
    }

    #[test]
    fn test_generate_register_report() {
        let (_temp_dir, storage) = create_test_storage();

        // Create test data
        let account = Account::with_starting_balance(
            "Checking",
            AccountType::Checking,
            Money::from_cents(100000),
        );
        storage.accounts.upsert(account.clone()).unwrap();
        storage.accounts.save().unwrap();

        let group = CategoryGroup::new("Test");
        storage.categories.upsert_group(group.clone()).unwrap();
        let cat = Category::new("Groceries", group.id);
        storage.categories.upsert_category(cat.clone()).unwrap();
        storage.categories.save().unwrap();

        // Add transactions
        let mut txn1 = Transaction::new(
            account.id,
            NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            Money::from_cents(-5000),
        );
        txn1.payee_name = "Grocery Store".to_string();
        txn1.category_id = Some(cat.id);
        storage.transactions.upsert(txn1).unwrap();

        let txn2 = Transaction::new(
            account.id,
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            Money::from_cents(200000),
        );
        storage.transactions.upsert(txn2).unwrap();

        // Generate report
        let report = AccountRegisterReport::generate(
            &storage,
            account.id,
            RegisterFilter::default(),
        )
        .unwrap();

        assert_eq!(report.entries.len(), 2);
        assert_eq!(report.starting_balance.cents(), 100000);
        // 100000 - 5000 + 200000 = 295000
        assert_eq!(report.ending_balance.cents(), 295000);
    }

    #[test]
    fn test_register_filter() {
        let (_temp_dir, storage) = create_test_storage();

        let account = Account::new("Checking", AccountType::Checking);
        storage.accounts.upsert(account.clone()).unwrap();

        // Add transactions on different dates
        for day in 1..10 {
            let txn = Transaction::new(
                account.id,
                NaiveDate::from_ymd_opt(2025, 1, day).unwrap(),
                Money::from_cents(-1000),
            );
            storage.transactions.upsert(txn).unwrap();
        }

        // Filter by date range
        let filter = RegisterFilter {
            start_date: Some(NaiveDate::from_ymd_opt(2025, 1, 3).unwrap()),
            end_date: Some(NaiveDate::from_ymd_opt(2025, 1, 7).unwrap()),
            ..Default::default()
        };

        let report =
            AccountRegisterReport::generate(&storage, account.id, filter).unwrap();

        assert_eq!(report.entries.len(), 5); // Days 3, 4, 5, 6, 7
    }

    #[test]
    fn test_csv_export() {
        let (_temp_dir, storage) = create_test_storage();

        let account = Account::new("Checking", AccountType::Checking);
        storage.accounts.upsert(account.clone()).unwrap();

        let mut txn = Transaction::new(
            account.id,
            NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            Money::from_cents(-5000),
        );
        txn.payee_name = "Test Payee".to_string();
        storage.transactions.upsert(txn).unwrap();

        let report = AccountRegisterReport::generate(
            &storage,
            account.id,
            RegisterFilter::default(),
        )
        .unwrap();

        let mut csv_output = Vec::new();
        report.export_csv(&mut csv_output).unwrap();

        let csv_string = String::from_utf8(csv_output).unwrap();
        assert!(csv_string.contains("Account,Date,Payee,Category,Memo,Amount"));
        assert!(csv_string.contains("Test Payee"));
    }
}
