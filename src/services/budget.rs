//! Budget service
//!
//! Provides business logic for budget management including allocation,
//! Available to Budget calculation, and budget overview.

use crate::audit::EntityType;
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{BudgetAllocation, BudgetPeriod, CategoryBudgetSummary, CategoryId, Money};
use crate::services::CategoryService;
use crate::storage::Storage;

/// Service for budget management
pub struct BudgetService<'a> {
    storage: &'a Storage,
}

/// Budget overview for a period
#[derive(Debug, Clone)]
pub struct BudgetOverview {
    pub period: BudgetPeriod,
    pub total_budgeted: Money,
    pub total_activity: Money,
    pub total_available: Money,
    pub available_to_budget: Money,
    pub categories: Vec<CategoryBudgetSummary>,
}

impl<'a> BudgetService<'a> {
    /// Create a new budget service
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Assign funds to a category for a period
    pub fn assign_to_category(
        &self,
        category_id: CategoryId,
        period: &BudgetPeriod,
        amount: Money,
    ) -> EnvelopeResult<BudgetAllocation> {
        // Verify category exists
        let category = self
            .storage
            .categories
            .get_category(category_id)?
            .ok_or_else(|| EnvelopeError::category_not_found(category_id.to_string()))?;

        // Get or create allocation
        let mut allocation = self.storage.budget.get_or_default(category_id, period)?;
        let before = allocation.clone();

        allocation.set_budgeted(amount);

        // Validate
        allocation
            .validate()
            .map_err(|e| EnvelopeError::Budget(e.to_string()))?;

        // Save
        self.storage.budget.upsert(allocation.clone())?;
        self.storage.budget.save()?;

        // Audit
        self.storage.log_update(
            EntityType::BudgetAllocation,
            format!("{}:{}", category_id, period),
            Some(category.name),
            &before,
            &allocation,
            Some(format!("budgeted: {} -> {}", before.budgeted, allocation.budgeted)),
        )?;

        Ok(allocation)
    }

    /// Add to a category's budget for a period
    pub fn add_to_category(
        &self,
        category_id: CategoryId,
        period: &BudgetPeriod,
        amount: Money,
    ) -> EnvelopeResult<BudgetAllocation> {
        // Verify category exists
        let category = self
            .storage
            .categories
            .get_category(category_id)?
            .ok_or_else(|| EnvelopeError::category_not_found(category_id.to_string()))?;

        // Get or create allocation
        let mut allocation = self.storage.budget.get_or_default(category_id, period)?;
        let before = allocation.clone();

        allocation.add_budgeted(amount);

        // Validate (check not negative)
        allocation
            .validate()
            .map_err(|e| EnvelopeError::Budget(e.to_string()))?;

        // Save
        self.storage.budget.upsert(allocation.clone())?;
        self.storage.budget.save()?;

        // Audit
        self.storage.log_update(
            EntityType::BudgetAllocation,
            format!("{}:{}", category_id, period),
            Some(category.name),
            &before,
            &allocation,
            Some(format!("budgeted: {} -> {} (+{})", before.budgeted, allocation.budgeted, amount)),
        )?;

        Ok(allocation)
    }

    /// Move funds between categories for a period
    pub fn move_between_categories(
        &self,
        from_category_id: CategoryId,
        to_category_id: CategoryId,
        period: &BudgetPeriod,
        amount: Money,
    ) -> EnvelopeResult<()> {
        if amount.is_zero() {
            return Ok(());
        }

        if amount.is_negative() {
            return Err(EnvelopeError::Budget("Amount to move must be positive".into()));
        }

        // Verify both categories exist
        let from_category = self
            .storage
            .categories
            .get_category(from_category_id)?
            .ok_or_else(|| EnvelopeError::category_not_found(from_category_id.to_string()))?;

        let to_category = self
            .storage
            .categories
            .get_category(to_category_id)?
            .ok_or_else(|| EnvelopeError::category_not_found(to_category_id.to_string()))?;

        // Get current allocations
        let mut from_alloc = self.storage.budget.get_or_default(from_category_id, period)?;
        let mut to_alloc = self.storage.budget.get_or_default(to_category_id, period)?;

        let from_before = from_alloc.clone();
        let to_before = to_alloc.clone();

        // Check if from has enough budgeted
        if from_alloc.budgeted < amount {
            return Err(EnvelopeError::InsufficientFunds {
                category: from_category.name.clone(),
                needed: amount.cents(),
                available: from_alloc.budgeted.cents(),
            });
        }

        // Move funds
        from_alloc.add_budgeted(-amount);
        to_alloc.add_budgeted(amount);

        // Validate both
        from_alloc
            .validate()
            .map_err(|e| EnvelopeError::Budget(e.to_string()))?;
        to_alloc
            .validate()
            .map_err(|e| EnvelopeError::Budget(e.to_string()))?;

        // Save both
        self.storage.budget.upsert(from_alloc.clone())?;
        self.storage.budget.upsert(to_alloc.clone())?;
        self.storage.budget.save()?;

        // Audit
        self.storage.log_update(
            EntityType::BudgetAllocation,
            format!("{}:{}", from_category_id, period),
            Some(from_category.name.clone()),
            &from_before,
            &from_alloc,
            Some(format!(
                "moved {} to '{}'",
                amount, to_category.name
            )),
        )?;

        self.storage.log_update(
            EntityType::BudgetAllocation,
            format!("{}:{}", to_category_id, period),
            Some(to_category.name.clone()),
            &to_before,
            &to_alloc,
            Some(format!(
                "received {} from '{}'",
                amount, from_category.name
            )),
        )?;

        Ok(())
    }

    /// Get the allocation for a category in a period
    pub fn get_allocation(
        &self,
        category_id: CategoryId,
        period: &BudgetPeriod,
    ) -> EnvelopeResult<BudgetAllocation> {
        self.storage.budget.get_or_default(category_id, period)
    }

    /// Get budget summary for a category in a period
    pub fn get_category_summary(
        &self,
        category_id: CategoryId,
        period: &BudgetPeriod,
    ) -> EnvelopeResult<CategoryBudgetSummary> {
        let allocation = self.storage.budget.get_or_default(category_id, period)?;

        // Calculate activity (sum of transactions in this category for this period)
        let activity = self.calculate_category_activity(category_id, period)?;

        Ok(CategoryBudgetSummary::from_allocation(&allocation, activity))
    }

    /// Calculate activity (spending) for a category in a period
    pub fn calculate_category_activity(
        &self,
        category_id: CategoryId,
        period: &BudgetPeriod,
    ) -> EnvelopeResult<Money> {
        let transactions = self.storage.transactions.get_by_category(category_id)?;

        // Filter to transactions within the period
        let period_start = period.start_date();
        let period_end = period.end_date();

        let activity: Money = transactions
            .iter()
            .filter(|t| t.date >= period_start && t.date <= period_end)
            .map(|t| {
                // Check if this is a split transaction
                if t.is_split() {
                    // Sum only the splits for this category
                    t.splits
                        .iter()
                        .filter(|s| s.category_id == category_id)
                        .map(|s| s.amount)
                        .sum()
                } else {
                    t.amount
                }
            })
            .sum();

        Ok(activity)
    }

    /// Calculate total income for a period (sum of all positive transactions)
    pub fn calculate_income_for_period(&self, period: &BudgetPeriod) -> EnvelopeResult<Money> {
        let period_start = period.start_date();
        let period_end = period.end_date();

        let transactions = self
            .storage
            .transactions
            .get_by_date_range(period_start, period_end)?;

        let income: Money = transactions
            .iter()
            .filter(|t| t.amount.is_positive())
            .map(|t| t.amount)
            .sum();

        Ok(income)
    }

    /// Calculate Available to Budget for a period
    ///
    /// Available to Budget = Total On-Budget Balances - Total Budgeted for current + prior periods
    pub fn get_available_to_budget(&self, period: &BudgetPeriod) -> EnvelopeResult<Money> {
        // Get total balance across all on-budget accounts
        let account_service = crate::services::AccountService::new(self.storage);
        let total_balance = account_service.total_on_budget_balance()?;

        // Get total budgeted for this period
        let allocations = self.storage.budget.get_for_period(period)?;
        let total_budgeted: Money = allocations.iter().map(|a| a.budgeted).sum();

        Ok(total_balance - total_budgeted)
    }

    /// Get a complete budget overview for a period
    pub fn get_budget_overview(&self, period: &BudgetPeriod) -> EnvelopeResult<BudgetOverview> {
        let category_service = CategoryService::new(self.storage);
        let categories = category_service.list_categories()?;

        let mut summaries = Vec::with_capacity(categories.len());
        let mut total_budgeted = Money::zero();
        let mut total_activity = Money::zero();
        let mut total_available = Money::zero();

        for category in &categories {
            let summary = self.get_category_summary(category.id, period)?;
            total_budgeted += summary.budgeted;
            total_activity += summary.activity;
            total_available += summary.available;
            summaries.push(summary);
        }

        let available_to_budget = self.get_available_to_budget(period)?;

        Ok(BudgetOverview {
            period: period.clone(),
            total_budgeted,
            total_activity,
            total_available,
            available_to_budget,
            categories: summaries,
        })
    }

    /// Get all allocations for a category (history)
    pub fn get_allocation_history(
        &self,
        category_id: CategoryId,
    ) -> EnvelopeResult<Vec<BudgetAllocation>> {
        self.storage.budget.get_for_category(category_id)
    }

    /// Calculate the carryover amount for a category going into a specific period
    ///
    /// This is the "Available" balance from the previous period, which includes:
    /// - Budgeted amount
    /// - Previous carryover
    /// - Activity (spending)
    pub fn get_carryover(
        &self,
        category_id: CategoryId,
        period: &BudgetPeriod,
    ) -> EnvelopeResult<Money> {
        let prev_period = period.prev();
        let summary = self.get_category_summary(category_id, &prev_period)?;
        Ok(summary.rollover_amount())
    }

    /// Apply rollover from the previous period to a category's allocation
    ///
    /// This should be called when entering a new period to carry forward
    /// any surplus or deficit from the previous period.
    pub fn apply_rollover(
        &self,
        category_id: CategoryId,
        period: &BudgetPeriod,
    ) -> EnvelopeResult<BudgetAllocation> {
        // Calculate carryover from previous period
        let carryover = self.get_carryover(category_id, period)?;

        // Get or create allocation for this period
        let mut allocation = self.storage.budget.get_or_default(category_id, period)?;

        // Only apply if carryover changed
        if allocation.carryover != carryover {
            let before = allocation.clone();
            allocation.set_carryover(carryover);

            // Save
            self.storage.budget.upsert(allocation.clone())?;
            self.storage.budget.save()?;

            // Get category name for audit
            let category = self.storage.categories.get_category(category_id)?;
            let category_name = category.map(|c| c.name);

            // Audit
            self.storage.log_update(
                EntityType::BudgetAllocation,
                format!("{}:{}", category_id, period),
                category_name,
                &before,
                &allocation,
                Some(format!("carryover: {} -> {}", before.carryover, allocation.carryover)),
            )?;
        }

        Ok(allocation)
    }

    /// Apply rollover for all categories for a period
    ///
    /// This calculates and sets the carryover amount for every category
    /// based on their Available balance from the previous period.
    pub fn apply_rollover_all(&self, period: &BudgetPeriod) -> EnvelopeResult<Vec<BudgetAllocation>> {
        let category_service = CategoryService::new(self.storage);
        let categories = category_service.list_categories()?;

        let mut allocations = Vec::with_capacity(categories.len());
        for category in &categories {
            let allocation = self.apply_rollover(category.id, period)?;
            allocations.push(allocation);
        }

        Ok(allocations)
    }

    /// Get a list of overspent categories for a period
    pub fn get_overspent_categories(&self, period: &BudgetPeriod) -> EnvelopeResult<Vec<CategoryBudgetSummary>> {
        let category_service = CategoryService::new(self.storage);
        let categories = category_service.list_categories()?;

        let mut overspent = Vec::new();
        for category in &categories {
            let summary = self.get_category_summary(category.id, period)?;
            if summary.is_overspent() {
                overspent.push(summary);
            }
        }

        Ok(overspent)
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

    fn setup_test_data(storage: &Storage) -> (CategoryId, CategoryId, BudgetPeriod) {
        // Create a group
        let group = CategoryGroup::new("Test Group");
        storage.categories.upsert_group(group.clone()).unwrap();

        // Create two categories
        let cat1 = Category::new("Groceries", group.id);
        let cat2 = Category::new("Dining Out", group.id);
        let cat1_id = cat1.id;
        let cat2_id = cat2.id;
        storage.categories.upsert_category(cat1).unwrap();
        storage.categories.upsert_category(cat2).unwrap();
        storage.categories.save().unwrap();

        let period = BudgetPeriod::monthly(2025, 1);

        (cat1_id, cat2_id, period)
    }

    #[test]
    fn test_assign_to_category() {
        let (_temp_dir, storage) = create_test_storage();
        let (cat_id, _, period) = setup_test_data(&storage);
        let service = BudgetService::new(&storage);

        let allocation = service
            .assign_to_category(cat_id, &period, Money::from_cents(50000))
            .unwrap();

        assert_eq!(allocation.budgeted.cents(), 50000);
    }

    #[test]
    fn test_add_to_category() {
        let (_temp_dir, storage) = create_test_storage();
        let (cat_id, _, period) = setup_test_data(&storage);
        let service = BudgetService::new(&storage);

        // First assignment
        service
            .assign_to_category(cat_id, &period, Money::from_cents(30000))
            .unwrap();

        // Add more
        let allocation = service
            .add_to_category(cat_id, &period, Money::from_cents(20000))
            .unwrap();

        assert_eq!(allocation.budgeted.cents(), 50000);
    }

    #[test]
    fn test_move_between_categories() {
        let (_temp_dir, storage) = create_test_storage();
        let (cat1_id, cat2_id, period) = setup_test_data(&storage);
        let service = BudgetService::new(&storage);

        // Assign to first category
        service
            .assign_to_category(cat1_id, &period, Money::from_cents(50000))
            .unwrap();

        // Move some to second
        service
            .move_between_categories(cat1_id, cat2_id, &period, Money::from_cents(20000))
            .unwrap();

        let alloc1 = service.get_allocation(cat1_id, &period).unwrap();
        let alloc2 = service.get_allocation(cat2_id, &period).unwrap();

        assert_eq!(alloc1.budgeted.cents(), 30000);
        assert_eq!(alloc2.budgeted.cents(), 20000);
    }

    #[test]
    fn test_move_insufficient_funds() {
        let (_temp_dir, storage) = create_test_storage();
        let (cat1_id, cat2_id, period) = setup_test_data(&storage);
        let service = BudgetService::new(&storage);

        // Assign to first category
        service
            .assign_to_category(cat1_id, &period, Money::from_cents(10000))
            .unwrap();

        // Try to move more than available
        let result =
            service.move_between_categories(cat1_id, cat2_id, &period, Money::from_cents(20000));

        assert!(matches!(result, Err(EnvelopeError::InsufficientFunds { .. })));
    }

    #[test]
    fn test_category_activity() {
        let (_temp_dir, storage) = create_test_storage();
        let (cat_id, _, period) = setup_test_data(&storage);

        // Create an account and add a transaction
        let account = Account::new("Checking", AccountType::Checking);
        storage.accounts.upsert(account.clone()).unwrap();

        let mut txn = Transaction::new(
            account.id,
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            Money::from_cents(-5000),
        );
        txn.category_id = Some(cat_id);
        storage.transactions.upsert(txn).unwrap();

        let service = BudgetService::new(&storage);
        let activity = service.calculate_category_activity(cat_id, &period).unwrap();

        assert_eq!(activity.cents(), -5000);
    }

    #[test]
    fn test_available_to_budget() {
        let (_temp_dir, storage) = create_test_storage();
        let (cat_id, _, period) = setup_test_data(&storage);

        // Create account with balance
        let account = Account::with_starting_balance("Checking", AccountType::Checking, Money::from_cents(100000));
        storage.accounts.upsert(account.clone()).unwrap();
        storage.accounts.save().unwrap();

        let service = BudgetService::new(&storage);

        // Before budgeting
        let atb = service.get_available_to_budget(&period).unwrap();
        assert_eq!(atb.cents(), 100000);

        // After budgeting $500
        service
            .assign_to_category(cat_id, &period, Money::from_cents(50000))
            .unwrap();

        let atb = service.get_available_to_budget(&period).unwrap();
        assert_eq!(atb.cents(), 50000); // 100000 - 50000
    }

    #[test]
    fn test_positive_carryover() {
        let (_temp_dir, storage) = create_test_storage();
        let (cat_id, _, jan) = setup_test_data(&storage);
        let feb = jan.next();

        let service = BudgetService::new(&storage);

        // Budget $500 in January, spend nothing
        service
            .assign_to_category(cat_id, &jan, Money::from_cents(50000))
            .unwrap();

        // Get carryover for February (should be $500 - $0 = $500)
        let carryover = service.get_carryover(cat_id, &feb).unwrap();
        assert_eq!(carryover.cents(), 50000);
    }

    #[test]
    fn test_negative_carryover() {
        let (_temp_dir, storage) = create_test_storage();
        let (cat_id, _, jan) = setup_test_data(&storage);
        let feb = jan.next();

        // Create account and add an overspending transaction
        let account = Account::new("Checking", AccountType::Checking);
        storage.accounts.upsert(account.clone()).unwrap();

        let mut txn = Transaction::new(
            account.id,
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            Money::from_cents(-60000), // Spent $600
        );
        txn.category_id = Some(cat_id);
        storage.transactions.upsert(txn).unwrap();

        let service = BudgetService::new(&storage);

        // Budget $500 in January, spent $600 (overspent by $100)
        service
            .assign_to_category(cat_id, &jan, Money::from_cents(50000))
            .unwrap();

        // Get carryover for February (should be $500 - $600 = -$100)
        let carryover = service.get_carryover(cat_id, &feb).unwrap();
        assert_eq!(carryover.cents(), -10000);
    }

    #[test]
    fn test_apply_rollover() {
        let (_temp_dir, storage) = create_test_storage();
        let (cat_id, _, jan) = setup_test_data(&storage);
        let feb = jan.next();

        let service = BudgetService::new(&storage);

        // Budget $500 in January
        service
            .assign_to_category(cat_id, &jan, Money::from_cents(50000))
            .unwrap();

        // Apply rollover to February
        let feb_alloc = service.apply_rollover(cat_id, &feb).unwrap();

        // Carryover should be $500
        assert_eq!(feb_alloc.carryover.cents(), 50000);
        assert_eq!(feb_alloc.budgeted.cents(), 0);
        assert_eq!(feb_alloc.total_budgeted().cents(), 50000);
    }

    #[test]
    fn test_apply_rollover_all() {
        let (_temp_dir, storage) = create_test_storage();
        let (cat1_id, cat2_id, jan) = setup_test_data(&storage);
        let feb = jan.next();

        let service = BudgetService::new(&storage);

        // Budget in January
        service
            .assign_to_category(cat1_id, &jan, Money::from_cents(50000))
            .unwrap();
        service
            .assign_to_category(cat2_id, &jan, Money::from_cents(20000))
            .unwrap();

        // Apply rollover for all categories
        let allocations = service.apply_rollover_all(&feb).unwrap();
        assert_eq!(allocations.len(), 2);

        // Check carryovers
        let cat1_alloc = service.get_allocation(cat1_id, &feb).unwrap();
        let cat2_alloc = service.get_allocation(cat2_id, &feb).unwrap();

        assert_eq!(cat1_alloc.carryover.cents(), 50000);
        assert_eq!(cat2_alloc.carryover.cents(), 20000);
    }

    #[test]
    fn test_overspent_categories() {
        let (_temp_dir, storage) = create_test_storage();
        let (cat1_id, cat2_id, period) = setup_test_data(&storage);

        // Create account and add overspending transaction to cat1
        let account = Account::new("Checking", AccountType::Checking);
        storage.accounts.upsert(account.clone()).unwrap();

        let mut txn = Transaction::new(
            account.id,
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            Money::from_cents(-60000), // Overspent in cat1
        );
        txn.category_id = Some(cat1_id);
        storage.transactions.upsert(txn).unwrap();

        let service = BudgetService::new(&storage);

        // Budget $500 in cat1 (will be overspent by $100)
        service
            .assign_to_category(cat1_id, &period, Money::from_cents(50000))
            .unwrap();

        // Budget $200 in cat2 (not overspent)
        service
            .assign_to_category(cat2_id, &period, Money::from_cents(20000))
            .unwrap();

        let overspent = service.get_overspent_categories(&period).unwrap();
        assert_eq!(overspent.len(), 1);
        assert_eq!(overspent[0].category_id, cat1_id);
        assert_eq!(overspent[0].available.cents(), -10000);
    }
}
