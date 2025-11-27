//! Display formatting for terminal output
//!
//! Provides utilities for formatting data models for terminal display,
//! including tables, colors, and status indicators.

pub mod account;
pub mod category;
pub mod transaction;

pub use account::{format_account_details, format_account_list};
pub use category::{format_category_details, format_category_tree, format_group_details, format_group_list};
pub use transaction::{
    format_transaction_details, format_transaction_list_by_account, format_transaction_register,
    format_transaction_row, format_transaction_short,
};
