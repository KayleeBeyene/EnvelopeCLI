//! Core data models for EnvelopeCLI
//!
//! This module contains all the data structures that represent the budgeting
//! domain: accounts, transactions, categories, budget allocations, etc.

pub mod account;
pub mod budget;
pub mod category;
pub mod ids;
pub mod money;
pub mod payee;
pub mod period;
pub mod transaction;

pub use account::{Account, AccountType};
pub use budget::BudgetAllocation;
pub use category::{Category, CategoryGroup, DefaultCategoryGroup};
pub use ids::{AccountId, CategoryGroupId, CategoryId, PayeeId, TransactionId};
pub use money::Money;
pub use payee::Payee;
pub use period::BudgetPeriod;
pub use transaction::{Split, Transaction, TransactionStatus};
