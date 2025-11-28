//! Reports module for EnvelopeCLI
//!
//! Provides various financial reports including budget overview,
//! spending analysis, account registers, and net worth summaries.

pub mod account_register;
pub mod budget_overview;
pub mod net_worth;
pub mod spending;

pub use account_register::{AccountRegisterReport, RegisterEntry, RegisterFilter};
pub use budget_overview::{BudgetOverviewReport, CategoryReportRow, GroupReportRow};
pub use net_worth::{NetWorthReport, NetWorthSummary};
pub use spending::{SpendingByCategory, SpendingReport};
