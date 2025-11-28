//! Layout definitions for the TUI
//!
//! Defines the overall layout structure: sidebar, main panel, status bar.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Layout regions for the TUI
pub struct AppLayout {
    /// Sidebar area (accounts list, view switcher)
    pub sidebar: Rect,
    /// Main content area
    pub main: Rect,
    /// Status bar at the bottom
    pub status_bar: Rect,
}

impl AppLayout {
    /// Calculate layout from available area
    pub fn new(area: Rect) -> Self {
        // Split into main area and status bar
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Main area
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        // Split main area into sidebar and content
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30), // Sidebar (fixed width)
                Constraint::Min(40),    // Main content
            ])
            .split(vertical[0]);

        Self {
            sidebar: horizontal[0],
            main: horizontal[1],
            status_bar: vertical[1],
        }
    }
}

/// Layout for the sidebar
pub struct SidebarLayout {
    /// Title/header area
    pub header: Rect,
    /// Account list area
    pub accounts: Rect,
    /// View switcher area
    pub view_switcher: Rect,
}

impl SidebarLayout {
    /// Calculate sidebar layout
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(5),    // Accounts
                Constraint::Length(5), // View switcher
            ])
            .split(area);

        Self {
            header: chunks[0],
            accounts: chunks[1],
            view_switcher: chunks[2],
        }
    }
}

/// Layout for the main panel header
pub struct MainPanelLayout {
    /// Header area (title, period selector for budget)
    pub header: Rect,
    /// Content area
    pub content: Rect,
}

impl MainPanelLayout {
    /// Calculate main panel layout
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(3),    // Content
            ])
            .split(area);

        Self {
            header: chunks[0],
            content: chunks[1],
        }
    }
}

/// Layout for the budget view
pub struct BudgetLayout {
    /// Available to Budget header
    pub atb_header: Rect,
    /// Category table
    pub categories: Rect,
}

impl BudgetLayout {
    /// Calculate budget view layout
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // ATB header
                Constraint::Min(3),    // Categories
            ])
            .split(area);

        Self {
            atb_header: chunks[0],
            categories: chunks[1],
        }
    }
}

/// Create a centered rect for dialogs
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Create a fixed-size centered rect for dialogs
pub fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(r.width), height.min(r.height))
}
