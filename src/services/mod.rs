//! Service layer for EnvelopeCLI
//!
//! The service layer provides business logic on top of the storage layer,
//! handling validation, computed fields, and cross-entity operations.

pub mod account;
pub mod budget;
pub mod category;
pub mod period;

pub use account::AccountService;
pub use budget::BudgetService;
pub use category::CategoryService;
pub use period::PeriodService;
