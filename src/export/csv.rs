//! CSV Export functionality
//!
//! Exports transactions, budget allocations, and account data to CSV format.

use crate::error::EnvelopeResult;
use crate::models::{BudgetPeriod, TransactionStatus};
use crate::services::{AccountService, BudgetService, CategoryService};
use crate::storage::Storage;
use std::io::Write;

/// Export all transactions to CSV
pub fn export_transactions_csv<W: Write>(storage: &Storage, writer: &mut W) -> EnvelopeResult<()> {
    let category_service = CategoryService::new(storage);
    let account_service = AccountService::new(storage);

    // Build lookups
    let categories = category_service.list_categories()?;
    let category_names: std::collections::HashMap<_, _> = categories
        .iter()
        .map(|c| (c.id, c.name.clone()))
        .collect();

    let accounts = account_service.list(true)?;
    let account_names: std::collections::HashMap<_, _> = accounts
        .iter()
        .map(|a| (a.id, a.name.clone()))
        .collect();

    // Write header
    writeln!(
        writer,
        "ID,Date,Account,Payee,Category,Memo,Amount,Status,Is Split,Is Transfer"
    )
    .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

    // Get all transactions
    let transactions = storage.transactions.get_all()?;

    for txn in transactions {
        let account_name = account_names
            .get(&txn.account_id)
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());

        let category_name = if txn.is_transfer() {
            "Transfer".to_string()
        } else if txn.is_split() {
            "Split".to_string()
        } else if let Some(cat_id) = txn.category_id {
            category_names
                .get(&cat_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "".to_string()
        };

        let status = match txn.status {
            TransactionStatus::Pending => "Pending",
            TransactionStatus::Cleared => "Cleared",
            TransactionStatus::Reconciled => "Reconciled",
        };

        writeln!(
            writer,
            "{},{},{},{},{},{},{:.2},{},{},{}",
            txn.id,
            txn.date,
            escape_csv(&account_name),
            escape_csv(&txn.payee_name),
            escape_csv(&category_name),
            escape_csv(&txn.memo),
            txn.amount.cents() as f64 / 100.0,
            status,
            txn.is_split(),
            txn.is_transfer()
        )
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

        // If split transaction, also export split details
        if txn.is_split() {
            for split in &txn.splits {
                let split_cat_name = category_names
                    .get(&split.category_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string());

                writeln!(
                    writer,
                    "{}-split,{},{},{},{},{},{:.2},{},{},{}",
                    txn.id,
                    txn.date,
                    escape_csv(&account_name),
                    escape_csv(&txn.payee_name),
                    escape_csv(&split_cat_name),
                    escape_csv(&split.memo),
                    split.amount.cents() as f64 / 100.0,
                    status,
                    "true",
                    "false"
                )
                .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
            }
        }
    }

    Ok(())
}

/// Export budget allocations to CSV
pub fn export_allocations_csv<W: Write>(
    storage: &Storage,
    writer: &mut W,
    periods: Option<Vec<BudgetPeriod>>,
) -> EnvelopeResult<()> {
    let category_service = CategoryService::new(storage);
    let budget_service = BudgetService::new(storage);

    // Build category lookup
    let categories = category_service.list_categories()?;
    let groups = category_service.list_groups()?;

    let group_names: std::collections::HashMap<_, _> = groups
        .iter()
        .map(|g| (g.id, g.name.clone()))
        .collect();

    // Write header
    writeln!(
        writer,
        "Period,Group,Category,Budgeted,Carryover,Activity,Available"
    )
    .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

    // Determine which periods to export
    let periods_to_export = if let Some(p) = periods {
        p
    } else {
        // Export last 12 months by default
        let current = BudgetPeriod::current_month();
        (0..12).map(|i| {
            let mut p = current.clone();
            for _ in 0..i {
                p = p.prev();
            }
            p
        }).collect()
    };

    for period in periods_to_export {
        for category in &categories {
            let summary = budget_service.get_category_summary(category.id, &period)?;
            let group_name = group_names
                .get(&category.group_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());

            writeln!(
                writer,
                "{},{},{},{:.2},{:.2},{:.2},{:.2}",
                period,
                escape_csv(&group_name),
                escape_csv(&category.name),
                summary.budgeted.cents() as f64 / 100.0,
                summary.carryover.cents() as f64 / 100.0,
                summary.activity.cents() as f64 / 100.0,
                summary.available.cents() as f64 / 100.0
            )
            .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
        }
    }

    Ok(())
}

/// Export accounts to CSV
pub fn export_accounts_csv<W: Write>(storage: &Storage, writer: &mut W) -> EnvelopeResult<()> {
    let account_service = AccountService::new(storage);
    let summaries = account_service.list_with_balances(true)?;

    // Write header
    writeln!(
        writer,
        "ID,Name,Type,On Budget,Archived,Starting Balance,Current Balance,Cleared Balance,Uncleared Count"
    )
    .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

    for summary in summaries {
        writeln!(
            writer,
            "{},{},{:?},{},{},{:.2},{:.2},{:.2},{}",
            summary.account.id,
            escape_csv(&summary.account.name),
            summary.account.account_type,
            summary.account.on_budget,
            summary.account.archived,
            summary.account.starting_balance.cents() as f64 / 100.0,
            summary.balance.cents() as f64 / 100.0,
            summary.cleared_balance.cents() as f64 / 100.0,
            summary.uncleared_count
        )
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
    }

    Ok(())
}

/// Escape a string for CSV format
fn escape_csv(s: &str) -> String {
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
    use crate::models::{Account, AccountType, Category, CategoryGroup, Money, Transaction};
    use chrono::NaiveDate;
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, Storage) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        let mut storage = Storage::new(paths).unwrap();
        storage.load_all().unwrap();
        (temp_dir, storage)
    }

    #[test]
    fn test_export_transactions_csv() {
        let (_temp_dir, storage) = create_test_storage();

        // Create test data
        let account = Account::new("Checking", AccountType::Checking);
        storage.accounts.upsert(account.clone()).unwrap();
        storage.accounts.save().unwrap();

        let group = CategoryGroup::new("Test");
        storage.categories.upsert_group(group.clone()).unwrap();
        let cat = Category::new("Groceries", group.id);
        storage.categories.upsert_category(cat.clone()).unwrap();
        storage.categories.save().unwrap();

        let mut txn = Transaction::new(
            account.id,
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            Money::from_cents(-5000),
        );
        txn.payee_name = "Test Store".to_string();
        txn.category_id = Some(cat.id);
        storage.transactions.upsert(txn).unwrap();

        let mut csv_output = Vec::new();
        export_transactions_csv(&storage, &mut csv_output).unwrap();

        let csv_string = String::from_utf8(csv_output).unwrap();
        assert!(csv_string.contains("ID,Date,Account,Payee"));
        assert!(csv_string.contains("Test Store"));
        assert!(csv_string.contains("Groceries"));
    }

    #[test]
    fn test_export_accounts_csv() {
        let (_temp_dir, storage) = create_test_storage();

        let account = Account::with_starting_balance(
            "Checking",
            AccountType::Checking,
            Money::from_cents(100000),
        );
        storage.accounts.upsert(account).unwrap();
        storage.accounts.save().unwrap();

        let mut csv_output = Vec::new();
        export_accounts_csv(&storage, &mut csv_output).unwrap();

        let csv_string = String::from_utf8(csv_output).unwrap();
        assert!(csv_string.contains("ID,Name,Type"));
        assert!(csv_string.contains("Checking"));
        assert!(csv_string.contains("1000.00"));
    }
}
