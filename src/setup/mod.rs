//! First-run setup wizard
//!
//! Provides an interactive setup experience for first-time users
//! to configure accounts, categories, and preferences.

pub mod wizard;
pub mod steps;

pub use wizard::{SetupWizard, SetupResult};
