//! First-run setup wizard
//!
//! Provides an interactive setup experience for first-time users
//! to configure accounts, categories, and preferences.

pub mod steps;
pub mod wizard;

pub use wizard::{SetupResult, SetupWizard};
