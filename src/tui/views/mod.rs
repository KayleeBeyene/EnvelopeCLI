//! TUI Views module
//!
//! Contains all the main views: accounts, register, budget, reports, reconcile,
//! as well as the sidebar and status bar.

pub mod account_list;
pub mod budget;
pub mod reconcile;
pub mod register;
pub mod sidebar;
pub mod status_bar;

use ratatui::Frame;

use super::app::{ActiveDialog, ActiveView, App};
use super::dialogs;
use super::layout::AppLayout;

/// Render the entire application
pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = AppLayout::new(frame.area());

    // Render sidebar
    sidebar::render(frame, app, layout.sidebar);

    // Render main view based on active view
    match app.active_view {
        ActiveView::Accounts => {
            account_list::render_main(frame, app, layout.main);
        }
        ActiveView::Register => {
            register::render(frame, app, layout.main);
        }
        ActiveView::Budget => {
            budget::render(frame, app, layout.main);
        }
        ActiveView::Reports => {
            // Reports view placeholder
            render_placeholder(frame, layout.main, "Reports");
        }
        ActiveView::Reconcile => {
            reconcile::render(frame, app, layout.main);
        }
    }

    // Render status bar
    status_bar::render(frame, app, layout.status_bar);

    // Render dialog if active
    if app.has_dialog() {
        render_dialog(frame, app);
    }
}

/// Render active dialog
fn render_dialog(frame: &mut Frame, app: &mut App) {
    match &app.active_dialog {
        ActiveDialog::Help => {
            dialogs::help::render(frame, app);
        }
        ActiveDialog::CommandPalette => {
            dialogs::command_palette::render(frame, app);
        }
        ActiveDialog::Confirm(message) => {
            dialogs::confirm::render(frame, message);
        }
        ActiveDialog::AddTransaction | ActiveDialog::EditTransaction(_) => {
            dialogs::transaction::render(frame, app);
        }
        ActiveDialog::MoveFunds => {
            dialogs::move_funds::render(frame, app);
        }
        ActiveDialog::BulkCategorize => {
            dialogs::bulk_categorize::render(frame, app);
        }
        ActiveDialog::ReconcileStart => {
            dialogs::reconcile_start::render(frame, app);
        }
        ActiveDialog::UnlockConfirm(state) => {
            dialogs::unlock_confirm::render(frame, state);
        }
        ActiveDialog::Adjustment => {
            dialogs::adjustment::render(frame, app);
        }
        ActiveDialog::EditBudget => {
            dialogs::edit_budget::render(frame, app);
        }
        ActiveDialog::AddAccount | ActiveDialog::EditAccount(_) => {
            dialogs::account::render(frame, app);
        }
        ActiveDialog::AddCategory | ActiveDialog::EditCategory(_) => {
            dialogs::category::render(frame, app);
        }
        ActiveDialog::AddGroup => {
            dialogs::group::render(frame, app);
        }
        ActiveDialog::None => {}
    }
}

/// Render a placeholder for unimplemented views
fn render_placeholder(frame: &mut Frame, area: ratatui::layout::Rect, name: &str) {
    use ratatui::style::{Color, Style};
    use ratatui::widgets::{Block, Borders, Paragraph};

    let block = Block::default()
        .title(name)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let text = Paragraph::new(format!("{} view - Coming soon!", name))
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(text, area);
}
