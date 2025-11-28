//! Reusable widgets for the TUI
//!
//! Contains custom widgets for common UI elements

pub mod error_dialog;
pub mod input;
pub mod notification;

// Re-export commonly used widgets
pub use error_dialog::{ErrorDialog, ErrorInfo, error_dialog_area};
pub use input::TextInput;
pub use notification::{Notification, NotificationQueue, NotificationType, NotificationWidget};
