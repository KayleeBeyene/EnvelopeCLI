//! EnvelopeCLI - Terminal-based zero-based budgeting application
//!
//! This library provides the core functionality for the EnvelopeCLI budgeting
//! application. It implements a zero-based budgeting system similar to YNAB,
//! but designed for terminal users who prefer CLI and TUI interfaces.
//!
//! # Architecture
//!
//! The crate is organized into the following modules:
//!
//! - `config`: Configuration and path management
//! - `error`: Custom error types
//! - `models`: Core data models (accounts, transactions, categories, etc.)
//! - `storage`: JSON file storage layer
//! - `services`: Business logic layer
//! - `audit`: Audit logging system
//! - `backup`: Automatic backup management
//!
//! # Example
//!
//! ```rust,ignore
//! use envelope::config::{paths::EnvelopePaths, settings::Settings};
//!
//! let paths = EnvelopePaths::new()?;
//! let settings = Settings::load_or_create(&paths)?;
//! ```

pub mod audit;
pub mod backup;
pub mod cli;
pub mod config;
pub mod display;
pub mod error;
pub mod models;
pub mod services;
pub mod storage;
// pub mod tui;       // Step 17+
// pub mod import;    // Step 15+
// pub mod export;    // Step 30
// pub mod reports;   // Step 28+
// pub mod crypto;    // Step 31
// pub mod setup;     // Step 32

pub use error::EnvelopeError;
