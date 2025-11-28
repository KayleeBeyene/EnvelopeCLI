//! Reports module for EnvelopeCLI
//!
//! Provides various financial reports including budget overview,
//! spending analysis, account registers, and net worth summaries.

pub mod budget_overview;
pub mod spending;
pub mod account_register;
pub mod net_worth;

pub use budget_overview::{BudgetOverviewReport, CategoryReportRow, GroupReportRow};
pub use spending::{SpendingReport, SpendingByCategory};
pub use account_register::{AccountRegisterReport, RegisterEntry, RegisterFilter};
pub use net_worth::{NetWorthReport, NetWorthSummary};
