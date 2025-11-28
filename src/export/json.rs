//! JSON Export functionality
//!
//! Exports the complete database to JSON format with schema versioning.

use crate::error::EnvelopeResult;
use crate::models::{Account, BudgetAllocation, Category, CategoryGroup, Payee, Transaction};
use crate::storage::Storage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Current export schema version
pub const EXPORT_SCHEMA_VERSION: &str = "1.0.0";

/// Full database export structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullExport {
    /// Schema version for compatibility checking
    pub schema_version: String,

    /// Export timestamp
    pub exported_at: DateTime<Utc>,

    /// Application version that created the export
    pub app_version: String,

    /// All accounts
    pub accounts: Vec<Account>,

    /// All category groups
    pub category_groups: Vec<CategoryGroup>,

    /// All categories
    pub categories: Vec<Category>,

    /// All transactions
    pub transactions: Vec<Transaction>,

    /// All budget allocations
    pub allocations: Vec<BudgetAllocation>,

    /// All payees
    pub payees: Vec<Payee>,

    /// Export metadata
    pub metadata: ExportMetadata,
}

/// Export metadata for reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Total number of accounts
    pub account_count: usize,

    /// Total number of transactions
    pub transaction_count: usize,

    /// Total number of categories
    pub category_count: usize,

    /// Total number of allocations
    pub allocation_count: usize,

    /// Total number of payees
    pub payee_count: usize,

    /// Date range of transactions (earliest)
    pub earliest_transaction: Option<String>,

    /// Date range of transactions (latest)
    pub latest_transaction: Option<String>,
}

impl FullExport {
    /// Create a new full export from storage
    pub fn from_storage(storage: &Storage) -> EnvelopeResult<Self> {
        let accounts = storage.accounts.get_all()?;
        let category_groups = storage.categories.get_all_groups()?;
        let categories = storage.categories.get_all_categories()?;
        let transactions = storage.transactions.get_all()?;
        let allocations = storage.budget.get_all()?;
        let payees = storage.payees.get_all()?;

        // Calculate metadata
        let earliest_transaction = transactions
            .iter()
            .map(|t| t.date)
            .min()
            .map(|d| d.to_string());

        let latest_transaction = transactions
            .iter()
            .map(|t| t.date)
            .max()
            .map(|d| d.to_string());

        let metadata = ExportMetadata {
            account_count: accounts.len(),
            transaction_count: transactions.len(),
            category_count: categories.len(),
            allocation_count: allocations.len(),
            payee_count: payees.len(),
            earliest_transaction,
            latest_transaction,
        };

        Ok(Self {
            schema_version: EXPORT_SCHEMA_VERSION.to_string(),
            exported_at: Utc::now(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            accounts,
            category_groups,
            categories,
            transactions,
            allocations,
            payees,
            metadata,
        })
    }

    /// Validate the export structure
    pub fn validate(&self) -> Result<(), String> {
        // Check schema version
        if self.schema_version != EXPORT_SCHEMA_VERSION {
            return Err(format!(
                "Schema version mismatch: expected {}, got {}",
                EXPORT_SCHEMA_VERSION, self.schema_version
            ));
        }

        // Check referential integrity
        let account_ids: std::collections::HashSet<_> =
            self.accounts.iter().map(|a| a.id).collect();
        let category_ids: std::collections::HashSet<_> =
            self.categories.iter().map(|c| c.id).collect();
        let group_ids: std::collections::HashSet<_> =
            self.category_groups.iter().map(|g| g.id).collect();

        // Validate transactions reference valid accounts
        for txn in &self.transactions {
            if !account_ids.contains(&txn.account_id) {
                return Err(format!(
                    "Transaction {} references unknown account {}",
                    txn.id, txn.account_id
                ));
            }
            if let Some(cat_id) = txn.category_id {
                if !category_ids.contains(&cat_id) {
                    return Err(format!(
                        "Transaction {} references unknown category {}",
                        txn.id, cat_id
                    ));
                }
            }
        }

        // Validate categories reference valid groups
        for cat in &self.categories {
            if !group_ids.contains(&cat.group_id) {
                return Err(format!(
                    "Category {} references unknown group {}",
                    cat.id, cat.group_id
                ));
            }
        }

        // Validate allocations reference valid categories
        for alloc in &self.allocations {
            if !category_ids.contains(&alloc.category_id) {
                return Err(format!(
                    "Allocation for category {} references unknown category",
                    alloc.category_id
                ));
            }
        }

        Ok(())
    }
}

/// Export the full database to JSON
pub fn export_full_json<W: Write>(
    storage: &Storage,
    writer: &mut W,
    pretty: bool,
) -> EnvelopeResult<()> {
    let export = FullExport::from_storage(storage)?;

    if pretty {
        serde_json::to_writer_pretty(writer, &export)
    } else {
        serde_json::to_writer(writer, &export)
    }
    .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

    Ok(())
}

/// Import from a JSON export (for verification/restore)
pub fn import_from_json(json_str: &str) -> EnvelopeResult<FullExport> {
    let export: FullExport = serde_json::from_str(json_str)
        .map_err(|e| crate::error::EnvelopeError::Import(e.to_string()))?;

    // Validate the import
    export
        .validate()
        .map_err(crate::error::EnvelopeError::Import)?;

    Ok(export)
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
    fn test_full_export() {
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
        txn.category_id = Some(cat.id);
        storage.transactions.upsert(txn).unwrap();

        // Export
        let export = FullExport::from_storage(&storage).unwrap();

        assert_eq!(export.schema_version, EXPORT_SCHEMA_VERSION);
        assert_eq!(export.accounts.len(), 1);
        assert_eq!(export.categories.len(), 1);
        assert_eq!(export.transactions.len(), 1);
        assert!(export.validate().is_ok());
    }

    #[test]
    fn test_json_roundtrip() {
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

        // Export to JSON
        let mut json_output = Vec::new();
        export_full_json(&storage, &mut json_output, true).unwrap();

        let json_string = String::from_utf8(json_output).unwrap();

        // Import back
        let imported = import_from_json(&json_string).unwrap();

        assert_eq!(imported.accounts.len(), 1);
        assert_eq!(imported.accounts[0].name, "Checking");
    }

    #[test]
    fn test_metadata() {
        let (_temp_dir, storage) = create_test_storage();

        // Create accounts
        for i in 0..3 {
            let account = Account::new(format!("Account {}", i), AccountType::Checking);
            storage.accounts.upsert(account).unwrap();
        }
        storage.accounts.save().unwrap();

        let export = FullExport::from_storage(&storage).unwrap();

        assert_eq!(export.metadata.account_count, 3);
        assert_eq!(export.metadata.transaction_count, 0);
    }
}
