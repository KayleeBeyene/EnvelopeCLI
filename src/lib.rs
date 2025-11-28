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
pub mod crypto;     // Step 31: Encryption at Rest
pub mod display;
pub mod error;
pub mod export;     // Step 30: Full Data Export
pub mod models;
pub mod reports;    // Steps 28-29: Reports
pub mod services;
pub mod setup;      // Step 32: First-Run Setup Wizard
pub mod storage;
pub mod tui;        // Phase 4: TUI

pub use error::EnvelopeError;
