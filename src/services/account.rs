//! Account service
//!
//! Provides business logic for account management including CRUD operations,
//! balance calculation, and validation.

use crate::audit::EntityType;
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::models::{Account, AccountId, AccountType, Money, TransactionStatus};
use crate::storage::Storage;

/// Service for account management
pub struct AccountService<'a> {
    storage: &'a Storage,
}

/// Summary of an account with computed fields
#[derive(Debug, Clone)]
pub struct AccountSummary {
    pub account: Account,
    /// Current balance (starting balance + all transactions)
    pub balance: Money,
    /// Cleared balance (starting balance + cleared/reconciled transactions only)
    pub cleared_balance: Money,
    /// Number of uncleared transactions
    pub uncleared_count: usize,
}

impl<'a> AccountService<'a> {
    /// Create a new account service
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Create a new account
    pub fn create(
        &self,
        name: &str,
        account_type: AccountType,
        starting_balance: Money,
        on_budget: bool,
    ) -> EnvelopeResult<Account> {
        // Validate name is not empty
        let name = name.trim();
        if name.is_empty() {
            return Err(EnvelopeError::Validation(
                "Account name cannot be empty".into(),
            ));
        }

        // Check for duplicate name
        if self.storage.accounts.name_exists(name, None)? {
            return Err(EnvelopeError::Duplicate {
                entity_type: "Account",
                identifier: name.to_string(),
            });
        }

        // Create the account
        let mut account = Account::with_starting_balance(name, account_type, starting_balance);
        account.on_budget = on_budget;

        // Validate
        account
            .validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        // Save to storage
        self.storage.accounts.upsert(account.clone())?;
        self.storage.accounts.save()?;

        // Audit log
        self.storage.log_create(
            EntityType::Account,
            account.id.to_string(),
            Some(account.name.clone()),
            &account,
        )?;

        Ok(account)
    }

    /// Get an account by ID
    pub fn get(&self, id: AccountId) -> EnvelopeResult<Option<Account>> {
        self.storage.accounts.get(id)
    }

    /// Get an account by name (case-insensitive)
    pub fn get_by_name(&self, name: &str) -> EnvelopeResult<Option<Account>> {
        self.storage.accounts.get_by_name(name)
    }

    /// Find an account by name or ID string
    pub fn find(&self, identifier: &str) -> EnvelopeResult<Option<Account>> {
        // Try by name first
        if let Some(account) = self.storage.accounts.get_by_name(identifier)? {
            return Ok(Some(account));
        }

        // Try parsing as ID
        if let Ok(id) = identifier.parse::<AccountId>() {
            return self.storage.accounts.get(id);
        }

        Ok(None)
    }

    /// Get all accounts
    pub fn list(&self, include_archived: bool) -> EnvelopeResult<Vec<Account>> {
        if include_archived {
            self.storage.accounts.get_all()
        } else {
            self.storage.accounts.get_active()
        }
    }

    /// Get all accounts with their computed balances
    pub fn list_with_balances(
        &self,
        include_archived: bool,
    ) -> EnvelopeResult<Vec<AccountSummary>> {
        let accounts = self.list(include_archived)?;
        let mut summaries = Vec::with_capacity(accounts.len());

        for account in accounts {
            let summary = self.get_summary(&account)?;
            summaries.push(summary);
        }

        Ok(summaries)
    }

    /// Get account summary with computed balances
    pub fn get_summary(&self, account: &Account) -> EnvelopeResult<AccountSummary> {
        let transactions = self.storage.transactions.get_by_account(account.id)?;

        let mut balance = account.starting_balance;
        let mut cleared_balance = account.starting_balance;
        let mut uncleared_count = 0;

        for txn in &transactions {
            balance += txn.amount;

            match txn.status {
                TransactionStatus::Cleared | TransactionStatus::Reconciled => {
                    cleared_balance += txn.amount;
                }
                TransactionStatus::Pending => {
                    uncleared_count += 1;
                }
            }
        }

        Ok(AccountSummary {
            account: account.clone(),
            balance,
            cleared_balance,
            uncleared_count,
        })
    }

    /// Calculate the current balance for an account
    pub fn calculate_balance(&self, account_id: AccountId) -> EnvelopeResult<Money> {
        let account = self
            .storage
            .accounts
            .get(account_id)?
            .ok_or_else(|| EnvelopeError::account_not_found(account_id.to_string()))?;

        let transactions = self.storage.transactions.get_by_account(account_id)?;
        let transaction_total: Money = transactions.iter().map(|t| t.amount).sum();

        Ok(account.starting_balance + transaction_total)
    }

    /// Calculate the cleared balance for an account
    pub fn calculate_cleared_balance(&self, account_id: AccountId) -> EnvelopeResult<Money> {
        let account = self
            .storage
            .accounts
            .get(account_id)?
            .ok_or_else(|| EnvelopeError::account_not_found(account_id.to_string()))?;

        let transactions = self.storage.transactions.get_by_account(account_id)?;
        let cleared_total: Money = transactions
            .iter()
            .filter(|t| {
                matches!(
                    t.status,
                    TransactionStatus::Cleared | TransactionStatus::Reconciled
                )
            })
            .map(|t| t.amount)
            .sum();

        Ok(account.starting_balance + cleared_total)
    }

    /// Update an account
    pub fn update(&self, id: AccountId, name: Option<&str>) -> EnvelopeResult<Account> {
        let mut account = self
            .storage
            .accounts
            .get(id)?
            .ok_or_else(|| EnvelopeError::account_not_found(id.to_string()))?;

        let before = account.clone();

        // Update name if provided
        if let Some(new_name) = name {
            let new_name = new_name.trim();
            if new_name.is_empty() {
                return Err(EnvelopeError::Validation(
                    "Account name cannot be empty".into(),
                ));
            }

            // Check for duplicate name (excluding self)
            if self.storage.accounts.name_exists(new_name, Some(id))? {
                return Err(EnvelopeError::Duplicate {
                    entity_type: "Account",
                    identifier: new_name.to_string(),
                });
            }

            account.name = new_name.to_string();
        }

        account.updated_at = chrono::Utc::now();

        // Validate
        account
            .validate()
            .map_err(|e| EnvelopeError::Validation(e.to_string()))?;

        // Save
        self.storage.accounts.upsert(account.clone())?;
        self.storage.accounts.save()?;

        // Audit log
        let diff = if before.name != account.name {
            Some(format!("name: {} -> {}", before.name, account.name))
        } else {
            None
        };

        self.storage.log_update(
            EntityType::Account,
            account.id.to_string(),
            Some(account.name.clone()),
            &before,
            &account,
            diff,
        )?;

        Ok(account)
    }

    /// Archive an account (soft delete)
    pub fn archive(&self, id: AccountId) -> EnvelopeResult<Account> {
        let mut account = self
            .storage
            .accounts
            .get(id)?
            .ok_or_else(|| EnvelopeError::account_not_found(id.to_string()))?;

        if account.archived {
            return Err(EnvelopeError::Validation(
                "Account is already archived".into(),
            ));
        }

        let before = account.clone();
        account.archive();

        // Save
        self.storage.accounts.upsert(account.clone())?;
        self.storage.accounts.save()?;

        // Audit log
        self.storage.log_update(
            EntityType::Account,
            account.id.to_string(),
            Some(account.name.clone()),
            &before,
            &account,
            Some("archived: false -> true".to_string()),
        )?;

        Ok(account)
    }

    /// Unarchive an account
    pub fn unarchive(&self, id: AccountId) -> EnvelopeResult<Account> {
        let mut account = self
            .storage
            .accounts
            .get(id)?
            .ok_or_else(|| EnvelopeError::account_not_found(id.to_string()))?;

        if !account.archived {
            return Err(EnvelopeError::Validation("Account is not archived".into()));
        }

        let before = account.clone();
        account.unarchive();

        // Save
        self.storage.accounts.upsert(account.clone())?;
        self.storage.accounts.save()?;

        // Audit log
        self.storage.log_update(
            EntityType::Account,
            account.id.to_string(),
            Some(account.name.clone()),
            &before,
            &account,
            Some("archived: true -> false".to_string()),
        )?;

        Ok(account)
    }

    /// Get total balance across all on-budget accounts
    pub fn total_on_budget_balance(&self) -> EnvelopeResult<Money> {
        let accounts = self.storage.accounts.get_active()?;
        let mut total = Money::zero();

        for account in accounts {
            if account.on_budget {
                total += self.calculate_balance(account.id)?;
            }
        }

        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::EnvelopePaths;
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, Storage) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        let mut storage = Storage::new(paths).unwrap();
        storage.load_all().unwrap();
        (temp_dir, storage)
    }

    #[test]
    fn test_create_account() {
        let (_temp_dir, storage) = create_test_storage();
        let service = AccountService::new(&storage);

        let account = service
            .create(
                "Checking",
                AccountType::Checking,
                Money::from_cents(100000),
                true,
            )
            .unwrap();

        assert_eq!(account.name, "Checking");
        assert_eq!(account.account_type, AccountType::Checking);
        assert_eq!(account.starting_balance.cents(), 100000);
        assert!(account.on_budget);
    }

    #[test]
    fn test_create_duplicate_name() {
        let (_temp_dir, storage) = create_test_storage();
        let service = AccountService::new(&storage);

        service
            .create("Checking", AccountType::Checking, Money::zero(), true)
            .unwrap();

        // Try to create another with same name
        let result = service.create("Checking", AccountType::Savings, Money::zero(), true);
        assert!(matches!(result, Err(EnvelopeError::Duplicate { .. })));
    }

    #[test]
    fn test_find_account() {
        let (_temp_dir, storage) = create_test_storage();
        let service = AccountService::new(&storage);

        let created = service
            .create("My Checking", AccountType::Checking, Money::zero(), true)
            .unwrap();

        // Find by name
        let found = service.find("My Checking").unwrap().unwrap();
        assert_eq!(found.id, created.id);

        // Case insensitive
        let found = service.find("my checking").unwrap().unwrap();
        assert_eq!(found.id, created.id);
    }

    #[test]
    fn test_list_accounts() {
        let (_temp_dir, storage) = create_test_storage();
        let service = AccountService::new(&storage);

        service
            .create("Account 1", AccountType::Checking, Money::zero(), true)
            .unwrap();
        service
            .create("Account 2", AccountType::Savings, Money::zero(), true)
            .unwrap();

        let accounts = service.list(false).unwrap();
        assert_eq!(accounts.len(), 2);
    }

    #[test]
    fn test_archive_account() {
        let (_temp_dir, storage) = create_test_storage();
        let service = AccountService::new(&storage);

        let account = service
            .create("Test", AccountType::Checking, Money::zero(), true)
            .unwrap();

        let archived = service.archive(account.id).unwrap();
        assert!(archived.archived);

        // Should not appear in active list
        let active = service.list(false).unwrap();
        assert!(active.is_empty());

        // Should appear in all list
        let all = service.list(true).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_update_account() {
        let (_temp_dir, storage) = create_test_storage();
        let service = AccountService::new(&storage);

        let account = service
            .create("Old Name", AccountType::Checking, Money::zero(), true)
            .unwrap();

        let updated = service.update(account.id, Some("New Name")).unwrap();
        assert_eq!(updated.name, "New Name");
    }

    #[test]
    fn test_balance_calculation() {
        let (_temp_dir, storage) = create_test_storage();
        let service = AccountService::new(&storage);

        let account = service
            .create(
                "Test",
                AccountType::Checking,
                Money::from_cents(100000),
                true,
            )
            .unwrap();

        // Add some transactions
        use crate::models::Transaction;
        use chrono::NaiveDate;

        let txn1 = Transaction::new(
            account.id,
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            Money::from_cents(-5000),
        );
        storage.transactions.upsert(txn1).unwrap();

        let mut txn2 = Transaction::new(
            account.id,
            NaiveDate::from_ymd_opt(2025, 1, 16).unwrap(),
            Money::from_cents(20000),
        );
        txn2.clear();
        storage.transactions.upsert(txn2).unwrap();

        // Total balance = 100000 - 5000 + 20000 = 115000
        let balance = service.calculate_balance(account.id).unwrap();
        assert_eq!(balance.cents(), 115000);

        // Cleared balance = 100000 + 20000 = 120000 (pending txn not counted)
        let cleared = service.calculate_cleared_balance(account.id).unwrap();
        assert_eq!(cleared.cents(), 120000);
    }
}
