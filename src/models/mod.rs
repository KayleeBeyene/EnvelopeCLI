//! Core data models for EnvelopeCLI
//!
//! This module contains all the data structures that represent the budgeting
//! domain: accounts, transactions, categories, budget allocations, etc.

pub mod account;
pub mod budget;
pub mod category;
pub mod ids;
pub mod income;
pub mod money;
pub mod payee;
pub mod period;
pub mod target;
pub mod transaction;

pub use account::{Account, AccountType};
pub use budget::{BudgetAllocation, CategoryBudgetSummary};
pub use category::{Category, CategoryGroup, DefaultCategoryGroup};
pub use ids::{AccountId, CategoryGroupId, CategoryId, IncomeId, PayeeId, TransactionId};
pub use income::IncomeExpectation;
pub use money::Money;
pub use payee::Payee;
pub use period::BudgetPeriod;
pub use target::{BudgetTarget, BudgetTargetId, TargetCadence};
pub use transaction::{Split, Transaction, TransactionStatus};
