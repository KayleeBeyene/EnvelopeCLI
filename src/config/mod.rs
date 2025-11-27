//! Configuration module for EnvelopeCLI
//!
//! This module provides configuration management including:
//! - XDG-compliant path resolution
//! - User settings persistence
//! - Application preferences

pub mod paths;
pub mod settings;

pub use paths::EnvelopePaths;
pub use settings::Settings;
