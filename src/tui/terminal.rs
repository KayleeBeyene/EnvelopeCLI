//! Terminal setup and teardown
//!
//! This module handles initializing and restoring the terminal state,
//! including setting up the panic hook to restore the terminal on crash.

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::panic;

use crate::config::paths::EnvelopePaths;
use crate::config::settings::Settings;
use crate::storage::Storage;

use super::app::App;
use super::event::{Event, EventHandler};
use super::handler::handle_event;

/// Type alias for our terminal
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal for TUI mode
pub fn init_terminal() -> Result<Tui> {
    // Set up panic hook to restore terminal on panic
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal before printing panic info
        let _ = restore_terminal_impl();
        original_hook(panic_info);
    }));

    // Enable raw mode and enter alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // Create terminal
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

/// Restore the terminal to its original state
pub fn restore_terminal() -> Result<()> {
    restore_terminal_impl()?;
    Ok(())
}

/// Internal implementation of terminal restoration
fn restore_terminal_impl() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// Run the TUI application
pub fn run_tui(storage: &Storage, settings: &Settings, paths: &EnvelopePaths) -> Result<()> {
    // Initialize terminal
    let mut terminal = init_terminal()?;

    // Create app state
    let mut app = App::new(storage, settings, paths);

    // Initialize account selection
    if let Ok(accounts) = storage.accounts.get_active() {
        if let Some(first) = accounts.first() {
            app.selected_account = Some(first.id);
        }
    }

    // Create event handler
    let events = EventHandler::default();

    // Main event loop
    loop {
        // Render
        terminal.draw(|frame| {
            super::views::render(frame, &mut app);
        })?;

        // Handle events
        match events.next()? {
            Event::Key(key_event) => {
                handle_event(&mut app, Event::Key(key_event))?;
            }
            Event::Mouse(mouse_event) => {
                handle_event(&mut app, Event::Mouse(mouse_event))?;
            }
            Event::Resize(_, _) => {
                // Terminal will redraw automatically
            }
            Event::Tick => {
                // Clear transient status messages after some time
                // (could add a timestamp check here)
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    restore_terminal()?;

    Ok(())
}
