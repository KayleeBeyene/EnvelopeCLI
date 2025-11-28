//! Spending Report
//!
//! Generates spending analysis by category for a given date range.

use crate::error::EnvelopeResult;
use crate::models::{CategoryGroupId, CategoryId, Money};
use crate::services::CategoryService;
use crate::storage::Storage;
use chrono::NaiveDate;
use std::collections::HashMap;
use std::io::Write;

/// Spending breakdown by category
#[derive(Debug, Clone)]
pub struct SpendingByCategory {
    /// Category ID
    pub category_id: CategoryId,
    /// Category name
    pub category_name: String,
    /// Group ID
    pub group_id: CategoryGroupId,
    /// Group name
    pub group_name: String,
    /// Total spending (negative value)
    pub total_spending: Money,
    /// Number of transactions
    pub transaction_count: usize,
    /// Percentage of total spending
    pub percentage: f64,
}

/// Spending by group summary
#[derive(Debug, Clone)]
pub struct SpendingByGroup {
    /// Group ID
    pub group_id: CategoryGroupId,
    /// Group name
    pub group_name: String,
    /// Categories in this group with spending
    pub categories: Vec<SpendingByCategory>,
    /// Total spending for this group
    pub total_spending: Money,
    /// Transaction count for this group
    pub transaction_count: usize,
    /// Percentage of total spending
    pub percentage: f64,
}

/// Spending Report
#[derive(Debug, Clone)]
pub struct SpendingReport {
    /// Start date of the report
    pub start_date: NaiveDate,
    /// End date of the report
    pub end_date: NaiveDate,
    /// Spending by group
    pub groups: Vec<SpendingByGroup>,
    /// Total spending across all categories
    pub total_spending: Money,
    /// Total income in the period
    pub total_income: Money,
    /// Total transaction count
    pub total_transactions: usize,
    /// Uncategorized spending
    pub uncategorized_spending: Money,
    /// Uncategorized transaction count
    pub uncategorized_count: usize,
}

impl SpendingReport {
    /// Generate a spending report for a date range
    pub fn generate(
        storage: &Storage,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> EnvelopeResult<Self> {
        let category_service = CategoryService::new(storage);
        let groups = category_service.list_groups()?;
        let categories = category_service.list_categories()?;

        // Get transactions in date range
        let transactions = storage.transactions.get_by_date_range(start_date, end_date)?;

        // Build category lookup
        let _category_map: HashMap<CategoryId, _> = categories
            .iter()
            .map(|c| (c.id, c.clone()))
            .collect();

        let _group_map: HashMap<CategoryGroupId, _> = groups
            .iter()
            .map(|g| (g.id, g.clone()))
            .collect();

        // Aggregate spending by category
        let mut category_spending: HashMap<CategoryId, (Money, usize)> = HashMap::new();
        let mut uncategorized_spending = Money::zero();
        let mut uncategorized_count = 0;
        let mut total_income = Money::zero();
        let mut total_spending = Money::zero();

        for txn in &transactions {
            if txn.amount.is_positive() {
                total_income += txn.amount;
            } else {
                if txn.is_split() {
                    // Handle split transactions
                    for split in &txn.splits {
                        let entry = category_spending
                            .entry(split.category_id)
                            .or_insert((Money::zero(), 0));
                        entry.0 += split.amount;
                        entry.1 += 1;
                        total_spending += split.amount;
                    }
                } else if let Some(cat_id) = txn.category_id {
                    let entry = category_spending
                        .entry(cat_id)
                        .or_insert((Money::zero(), 0));
                    entry.0 += txn.amount;
                    entry.1 += 1;
                    total_spending += txn.amount;
                } else if !txn.is_transfer() {
                    // Uncategorized (excluding transfers)
                    uncategorized_spending += txn.amount;
                    uncategorized_count += 1;
                    total_spending += txn.amount;
                }
            }
        }

        // Calculate total absolute spending for percentages
        let total_abs_spending = total_spending.abs();

        // Build report by group
        let mut report_groups: Vec<SpendingByGroup> = Vec::new();

        for group in &groups {
            let mut group_spending = SpendingByGroup {
                group_id: group.id,
                group_name: group.name.clone(),
                categories: Vec::new(),
                total_spending: Money::zero(),
                transaction_count: 0,
                percentage: 0.0,
            };

            // Find categories in this group with spending
            for category in categories.iter().filter(|c| c.group_id == group.id) {
                if let Some((spending, count)) = category_spending.get(&category.id) {
                    if !spending.is_zero() {
                        let percentage = if total_abs_spending.is_zero() {
                            0.0
                        } else {
                            (spending.abs().cents() as f64 / total_abs_spending.cents() as f64) * 100.0
                        };

                        let cat_spending = SpendingByCategory {
                            category_id: category.id,
                            category_name: category.name.clone(),
                            group_id: group.id,
                            group_name: group.name.clone(),
                            total_spending: *spending,
                            transaction_count: *count,
                            percentage,
                        };

                        group_spending.total_spending += *spending;
                        group_spending.transaction_count += *count;
                        group_spending.categories.push(cat_spending);
                    }
                }
            }

            // Sort categories by spending (most spending first)
            group_spending
                .categories
                .sort_by(|a, b| a.total_spending.cmp(&b.total_spending));

            // Calculate group percentage
            group_spending.percentage = if total_abs_spending.is_zero() {
                0.0
            } else {
                (group_spending.total_spending.abs().cents() as f64
                    / total_abs_spending.cents() as f64)
                    * 100.0
            };

            // Only include groups with spending
            if !group_spending.total_spending.is_zero() {
                report_groups.push(group_spending);
            }
        }

        // Sort groups by spending
        report_groups.sort_by(|a, b| a.total_spending.cmp(&b.total_spending));

        Ok(Self {
            start_date,
            end_date,
            groups: report_groups,
            total_spending,
            total_income,
            total_transactions: transactions.len(),
            uncategorized_spending,
            uncategorized_count,
        })
    }

    /// Format the report for terminal display
    pub fn format_terminal(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "Spending Report: {} to {}\n",
            self.start_date, self.end_date
        ));
        output.push_str(&"=".repeat(80));
        output.push('\n');
        output.push_str(&format!("Total Spending: {}\n", self.total_spending.abs()));
        output.push_str(&format!("Total Income: {}\n", self.total_income));
        output.push_str(&format!("Total Transactions: {}\n\n", self.total_transactions));

        // Column headers
        output.push_str(&format!(
            "{:<35} {:>12} {:>8} {:>8}\n",
            "Category", "Amount", "Count", "%"
        ));
        output.push_str(&"-".repeat(80));
        output.push('\n');

        // Groups and categories
        for group in &self.groups {
            // Group header
            output.push_str(&format!(
                "\n{} ({:.1}%)\n",
                group.group_name.to_uppercase(),
                group.percentage
            ));

            for category in &group.categories {
                output.push_str(&format!(
                    "  {:<33} {:>12} {:>8} {:>7.1}%\n",
                    category.category_name,
                    category.total_spending.abs(),
                    category.transaction_count,
                    category.percentage
                ));
            }

            // Group total
            output.push_str(&format!(
                "  {:<33} {:>12} {:>8}\n",
                "Group Total:",
                group.total_spending.abs(),
                group.transaction_count
            ));
        }

        // Uncategorized
        if !self.uncategorized_spending.is_zero() {
            output.push_str(&format!(
                "\n{:<35} {:>12} {:>8}\n",
                "UNCATEGORIZED",
                self.uncategorized_spending.abs(),
                self.uncategorized_count
            ));
        }

        // Grand total
        output.push_str(&"-".repeat(80));
        output.push('\n');
        output.push_str(&format!(
            "{:<35} {:>12} {:>8}\n",
            "TOTAL SPENDING",
            self.total_spending.abs(),
            self.total_transactions
        ));

        output
    }

    /// Export the report to CSV format
    pub fn export_csv<W: Write>(&self, writer: &mut W) -> EnvelopeResult<()> {
        // Write header
        writeln!(
            writer,
            "Start Date,End Date,Group,Category,Amount,Transaction Count,Percentage"
        )
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

        // Write data rows
        for group in &self.groups {
            for category in &group.categories {
                writeln!(
                    writer,
                    "{},{},{},{},{:.2},{},{:.2}",
                    self.start_date,
                    self.end_date,
                    group.group_name,
                    category.category_name,
                    category.total_spending.abs().cents() as f64 / 100.0,
                    category.transaction_count,
                    category.percentage
                )
                .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
            }
        }

        // Uncategorized
        if !self.uncategorized_spending.is_zero() {
            writeln!(
                writer,
                "{},{},UNCATEGORIZED,,{:.2},{},",
                self.start_date,
                self.end_date,
                self.uncategorized_spending.abs().cents() as f64 / 100.0,
                self.uncategorized_count
            )
            .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
        }

        // Total row
        writeln!(
            writer,
            "{},{},TOTAL,,{:.2},{},100.00",
            self.start_date,
            self.end_date,
            self.total_spending.abs().cents() as f64 / 100.0,
            self.total_transactions
        )
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

        Ok(())
    }

    /// Get top spending categories
    pub fn top_categories(&self, limit: usize) -> Vec<&SpendingByCategory> {
        let mut all_categories: Vec<_> = self
            .groups
            .iter()
            .flat_map(|g| &g.categories)
            .collect();

        // Sort by spending (most spending first - remember spending is negative)
        all_categories.sort_by(|a, b| a.total_spending.cmp(&b.total_spending));

        all_categories.into_iter().take(limit).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::EnvelopePaths;
    use crate::models::{Account, AccountType, Category, CategoryGroup, Transaction};
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, Storage) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        let mut storage = Storage::new(paths).unwrap();
        storage.load_all().unwrap();
        (temp_dir, storage)
    }

    #[test]
    fn test_generate_spending_report() {
        let (_temp_dir, storage) = create_test_storage();

        // Create test data
        let group = CategoryGroup::new("Test Group");
        storage.categories.upsert_group(group.clone()).unwrap();

        let cat1 = Category::new("Groceries", group.id);
        let cat2 = Category::new("Dining Out", group.id);
        storage.categories.upsert_category(cat1.clone()).unwrap();
        storage.categories.upsert_category(cat2.clone()).unwrap();
        storage.categories.save().unwrap();

        let account = Account::new("Checking", AccountType::Checking);
        storage.accounts.upsert(account.clone()).unwrap();
        storage.accounts.save().unwrap();

        // Add transactions
        let mut txn1 = Transaction::new(
            account.id,
            NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            Money::from_cents(-5000),
        );
        txn1.category_id = Some(cat1.id);
        storage.transactions.upsert(txn1).unwrap();

        let mut txn2 = Transaction::new(
            account.id,
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            Money::from_cents(-3000),
        );
        txn2.category_id = Some(cat2.id);
        storage.transactions.upsert(txn2).unwrap();

        // Income transaction
        let txn3 = Transaction::new(
            account.id,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            Money::from_cents(200000),
        );
        storage.transactions.upsert(txn3).unwrap();

        // Generate report
        let report = SpendingReport::generate(
            &storage,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
        )
        .unwrap();

        assert_eq!(report.total_spending.cents(), -8000);
        assert_eq!(report.total_income.cents(), 200000);
        assert_eq!(report.groups.len(), 1);
        assert_eq!(report.groups[0].categories.len(), 2);
    }

    #[test]
    fn test_top_categories() {
        let (_temp_dir, storage) = create_test_storage();

        // Setup with multiple categories
        let group = CategoryGroup::new("Test");
        storage.categories.upsert_group(group.clone()).unwrap();

        let cats: Vec<_> = (0..5)
            .map(|i| {
                let cat = Category::new(format!("Category {}", i), group.id);
                storage.categories.upsert_category(cat.clone()).unwrap();
                cat
            })
            .collect();
        storage.categories.save().unwrap();

        let account = Account::new("Checking", AccountType::Checking);
        storage.accounts.upsert(account.clone()).unwrap();

        // Add varying spending amounts
        for (i, cat) in cats.iter().enumerate() {
            let mut txn = Transaction::new(
                account.id,
                NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
                Money::from_cents(-((i + 1) as i64 * 1000)),
            );
            txn.category_id = Some(cat.id);
            storage.transactions.upsert(txn).unwrap();
        }

        let report = SpendingReport::generate(
            &storage,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
        )
        .unwrap();

        let top = report.top_categories(3);
        assert_eq!(top.len(), 3);
        // Should be sorted by spending (highest spending first)
        assert!(top[0].total_spending.cents() <= top[1].total_spending.cents());
    }
}
