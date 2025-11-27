//! Terminal User Interface module
//!
//! This module provides a full-featured TUI for EnvelopeCLI using ratatui.
//! The TUI includes views for accounts, transactions, budget management,
//! and various dialogs for data entry.

pub mod app;
pub mod event;
pub mod handler;
pub mod terminal;

// Views
pub mod views;

// Widgets
pub mod widgets;

// Dialogs
pub mod dialogs;

// Layout
pub mod layout;

// Commands and keybindings
pub mod commands;
pub mod keybindings;

pub use app::App;
pub use terminal::run_tui;
