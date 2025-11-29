//! Service layer for EnvelopeCLI
//!
//! The service layer provides business logic on top of the storage layer,
//! handling validation, computed fields, and cross-entity operations.

pub mod account;
pub mod budget;
pub mod category;
pub mod import;
pub mod income;
pub mod payee;
pub mod period;
pub mod reconciliation;
pub mod transaction;
pub mod transfer;

pub use account::AccountService;
pub use budget::BudgetService;
pub use category::CategoryService;
pub use import::{
    ColumnMapping, ImportPreviewEntry, ImportResult, ImportService, ImportStatus, ParsedTransaction,
};
pub use income::IncomeService;
pub use payee::PayeeService;
pub use period::PeriodService;
pub use reconciliation::{
    ReconciliationResult, ReconciliationService, ReconciliationSession, ReconciliationSummary,
};
pub use transaction::{CreateTransactionInput, TransactionFilter, TransactionService};
pub use transfer::TransferService;
