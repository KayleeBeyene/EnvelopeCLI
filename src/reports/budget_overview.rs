//! Budget Overview Report
//!
//! Generates a comprehensive budget overview showing all categories
//! with budgeted, activity (spending), and available amounts.

use crate::error::EnvelopeResult;
use crate::models::{BudgetPeriod, CategoryGroupId, CategoryId, Money};
use crate::services::{BudgetService, CategoryService};
use crate::storage::Storage;
use std::io::Write;

/// A row in the budget report for a single category
#[derive(Debug, Clone)]
pub struct CategoryReportRow {
    /// Category ID
    pub category_id: CategoryId,
    /// Category name
    pub category_name: String,
    /// Group ID this category belongs to
    pub group_id: CategoryGroupId,
    /// Amount budgeted for this period
    pub budgeted: Money,
    /// Amount carried over from previous period
    pub carryover: Money,
    /// Activity (spending) for this period
    pub activity: Money,
    /// Available balance (budgeted + carryover + activity)
    pub available: Money,
}

impl CategoryReportRow {
    /// Check if this category is overspent
    pub fn is_overspent(&self) -> bool {
        self.available.is_negative()
    }
}

/// A row in the budget report for a category group with totals
#[derive(Debug, Clone)]
pub struct GroupReportRow {
    /// Group ID
    pub group_id: CategoryGroupId,
    /// Group name
    pub group_name: String,
    /// Categories in this group
    pub categories: Vec<CategoryReportRow>,
    /// Total budgeted for this group
    pub total_budgeted: Money,
    /// Total carryover for this group
    pub total_carryover: Money,
    /// Total activity for this group
    pub total_activity: Money,
    /// Total available for this group
    pub total_available: Money,
}

impl GroupReportRow {
    /// Create a new group row
    pub fn new(group_id: CategoryGroupId, group_name: String) -> Self {
        Self {
            group_id,
            group_name,
            categories: Vec::new(),
            total_budgeted: Money::zero(),
            total_carryover: Money::zero(),
            total_activity: Money::zero(),
            total_available: Money::zero(),
        }
    }

    /// Add a category to this group
    pub fn add_category(&mut self, category: CategoryReportRow) {
        self.total_budgeted += category.budgeted;
        self.total_carryover += category.carryover;
        self.total_activity += category.activity;
        self.total_available += category.available;
        self.categories.push(category);
    }

    /// Check if any category in this group is overspent
    pub fn has_overspent(&self) -> bool {
        self.categories.iter().any(|c| c.is_overspent())
    }
}

/// Budget Overview Report
#[derive(Debug, Clone)]
pub struct BudgetOverviewReport {
    /// The budget period for this report
    pub period: BudgetPeriod,
    /// Groups with their categories
    pub groups: Vec<GroupReportRow>,
    /// Grand total budgeted
    pub grand_total_budgeted: Money,
    /// Grand total carryover
    pub grand_total_carryover: Money,
    /// Grand total activity
    pub grand_total_activity: Money,
    /// Grand total available
    pub grand_total_available: Money,
    /// Available to Budget (funds not yet assigned)
    pub available_to_budget: Money,
}

impl BudgetOverviewReport {
    /// Generate a budget overview report for a period
    pub fn generate(storage: &Storage, period: &BudgetPeriod) -> EnvelopeResult<Self> {
        let budget_service = BudgetService::new(storage);
        let category_service = CategoryService::new(storage);

        // Get all groups and categories
        let groups = category_service.list_groups()?;
        let categories = category_service.list_categories()?;

        let mut report_groups: Vec<GroupReportRow> = Vec::new();
        let mut grand_total_budgeted = Money::zero();
        let mut grand_total_carryover = Money::zero();
        let mut grand_total_activity = Money::zero();
        let mut grand_total_available = Money::zero();

        // Build report by group
        for group in &groups {
            let mut group_row = GroupReportRow::new(group.id, group.name.clone());

            // Find categories in this group
            for category in categories.iter().filter(|c| c.group_id == group.id) {
                let summary = budget_service.get_category_summary(category.id, period)?;

                let category_row = CategoryReportRow {
                    category_id: category.id,
                    category_name: category.name.clone(),
                    group_id: group.id,
                    budgeted: summary.budgeted,
                    carryover: summary.carryover,
                    activity: summary.activity,
                    available: summary.available,
                };

                group_row.add_category(category_row);
            }

            // Add to grand totals
            grand_total_budgeted += group_row.total_budgeted;
            grand_total_carryover += group_row.total_carryover;
            grand_total_activity += group_row.total_activity;
            grand_total_available += group_row.total_available;

            report_groups.push(group_row);
        }

        // Calculate available to budget
        let available_to_budget = budget_service.get_available_to_budget(period)?;

        Ok(Self {
            period: period.clone(),
            groups: report_groups,
            grand_total_budgeted,
            grand_total_carryover,
            grand_total_activity,
            grand_total_available,
            available_to_budget,
        })
    }

    /// Format the report for terminal display
    pub fn format_terminal(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("Budget Overview - {}\n", self.period));
        output.push_str(&"=".repeat(80));
        output.push('\n');
        output.push_str(&format!(
            "Available to Budget: {}\n\n",
            self.available_to_budget
        ));

        // Column headers
        output.push_str(&format!(
            "{:<30} {:>12} {:>12} {:>12}\n",
            "Category", "Budgeted", "Activity", "Available"
        ));
        output.push_str(&"-".repeat(80));
        output.push('\n');

        // Groups and categories
        for group in &self.groups {
            // Group header
            output.push_str(&format!("\n{}\n", group.group_name.to_uppercase()));

            for category in &group.categories {
                let available_display = if category.is_overspent() {
                    format!("{} *", category.available)
                } else {
                    category.available.to_string()
                };

                output.push_str(&format!(
                    "  {:<28} {:>12} {:>12} {:>12}\n",
                    category.category_name, category.budgeted, category.activity, available_display
                ));
            }

            // Group total
            output.push_str(&format!(
                "  {:<28} {:>12} {:>12} {:>12}\n",
                "Group Total:", group.total_budgeted, group.total_activity, group.total_available
            ));
        }

        // Grand totals
        output.push_str(&"-".repeat(80));
        output.push('\n');
        output.push_str(&format!(
            "{:<30} {:>12} {:>12} {:>12}\n",
            "GRAND TOTAL",
            self.grand_total_budgeted,
            self.grand_total_activity,
            self.grand_total_available
        ));

        output.push_str("\n* = Overspent\n");

        output
    }

    /// Export the report to CSV format
    pub fn export_csv<W: Write>(&self, writer: &mut W) -> EnvelopeResult<()> {
        // Write header
        writeln!(
            writer,
            "Period,Group,Category,Budgeted,Carryover,Activity,Available"
        )
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

        // Write data rows
        for group in &self.groups {
            for category in &group.categories {
                writeln!(
                    writer,
                    "{},{},{},{:.2},{:.2},{:.2},{:.2}",
                    self.period,
                    group.group_name,
                    category.category_name,
                    category.budgeted.cents() as f64 / 100.0,
                    category.carryover.cents() as f64 / 100.0,
                    category.activity.cents() as f64 / 100.0,
                    category.available.cents() as f64 / 100.0,
                )
                .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
            }

            // Group total row
            writeln!(
                writer,
                "{},{},TOTAL,{:.2},{:.2},{:.2},{:.2}",
                self.period,
                group.group_name,
                group.total_budgeted.cents() as f64 / 100.0,
                group.total_carryover.cents() as f64 / 100.0,
                group.total_activity.cents() as f64 / 100.0,
                group.total_available.cents() as f64 / 100.0,
            )
            .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
        }

        // Grand total row
        writeln!(
            writer,
            "{},GRAND TOTAL,,{:.2},{:.2},{:.2},{:.2}",
            self.period,
            self.grand_total_budgeted.cents() as f64 / 100.0,
            self.grand_total_carryover.cents() as f64 / 100.0,
            self.grand_total_activity.cents() as f64 / 100.0,
            self.grand_total_available.cents() as f64 / 100.0,
        )
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

        Ok(())
    }

    /// Get count of overspent categories
    pub fn overspent_count(&self) -> usize {
        self.groups
            .iter()
            .flat_map(|g| &g.categories)
            .filter(|c| c.is_overspent())
            .count()
    }

    /// Get list of overspent categories
    pub fn overspent_categories(&self) -> Vec<&CategoryReportRow> {
        self.groups
            .iter()
            .flat_map(|g| &g.categories)
            .filter(|c| c.is_overspent())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::EnvelopePaths;
    use crate::models::{Account, AccountType, Category, CategoryGroup, Transaction};
    use chrono::NaiveDate;
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, Storage) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        let mut storage = Storage::new(paths).unwrap();
        storage.load_all().unwrap();
        (temp_dir, storage)
    }

    fn setup_test_data(storage: &Storage) -> BudgetPeriod {
        // Create a group
        let group = CategoryGroup::new("Test Group");
        storage.categories.upsert_group(group.clone()).unwrap();

        // Create categories
        let cat1 = Category::new("Groceries", group.id);
        let cat2 = Category::new("Dining Out", group.id);
        storage.categories.upsert_category(cat1.clone()).unwrap();
        storage.categories.upsert_category(cat2.clone()).unwrap();
        storage.categories.save().unwrap();

        // Create account with starting balance
        let account = Account::with_starting_balance(
            "Checking",
            AccountType::Checking,
            Money::from_cents(100000),
        );
        storage.accounts.upsert(account.clone()).unwrap();
        storage.accounts.save().unwrap();

        // Create budget allocations
        let period = BudgetPeriod::monthly(2025, 1);
        let budget_service = BudgetService::new(storage);
        budget_service
            .assign_to_category(cat1.id, &period, Money::from_cents(50000))
            .unwrap();
        budget_service
            .assign_to_category(cat2.id, &period, Money::from_cents(20000))
            .unwrap();

        // Add a transaction
        let mut txn = Transaction::new(
            account.id,
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            Money::from_cents(-3000),
        );
        txn.category_id = Some(cat1.id);
        storage.transactions.upsert(txn).unwrap();

        period
    }

    #[test]
    fn test_generate_report() {
        let (_temp_dir, storage) = create_test_storage();
        let period = setup_test_data(&storage);

        let report = BudgetOverviewReport::generate(&storage, &period).unwrap();

        assert_eq!(report.period, period);
        assert_eq!(report.groups.len(), 1);
        assert_eq!(report.groups[0].categories.len(), 2);
        assert_eq!(report.grand_total_budgeted.cents(), 70000);
    }

    #[test]
    fn test_csv_export() {
        let (_temp_dir, storage) = create_test_storage();
        let period = setup_test_data(&storage);

        let report = BudgetOverviewReport::generate(&storage, &period).unwrap();

        let mut csv_output = Vec::new();
        report.export_csv(&mut csv_output).unwrap();

        let csv_string = String::from_utf8(csv_output).unwrap();
        assert!(csv_string.contains("Period,Group,Category,Budgeted,Carryover,Activity,Available"));
        assert!(csv_string.contains("Groceries"));
        assert!(csv_string.contains("Dining Out"));
    }

    #[test]
    fn test_terminal_format() {
        let (_temp_dir, storage) = create_test_storage();
        let period = setup_test_data(&storage);

        let report = BudgetOverviewReport::generate(&storage, &period).unwrap();
        let output = report.format_terminal();

        assert!(output.contains("Budget Overview"));
        assert!(output.contains("Groceries"));
        assert!(output.contains("GRAND TOTAL"));
    }
}
