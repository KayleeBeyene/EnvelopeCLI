//! Toast notification widget
//!
//! Displays temporary notifications to the user.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

/// Type of notification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    /// Informational message
    Info,
    /// Success message
    Success,
    /// Warning message
    Warning,
    /// Error message
    Error,
}

impl NotificationType {
    /// Get the color for this notification type
    pub fn color(&self) -> Color {
        match self {
            Self::Info => Color::Blue,
            Self::Success => Color::Green,
            Self::Warning => Color::Yellow,
            Self::Error => Color::Red,
        }
    }

    /// Get the icon/prefix for this notification type
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Info => "i",
            Self::Success => "+",
            Self::Warning => "!",
            Self::Error => "x",
        }
    }

    /// Get the title for this notification type
    pub fn title(&self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Success => "Success",
            Self::Warning => "Warning",
            Self::Error => "Error",
        }
    }
}

/// A toast notification
#[derive(Debug, Clone)]
pub struct Notification {
    /// The notification message
    pub message: String,
    /// Type of notification
    pub notification_type: NotificationType,
    /// Time when notification was created (for auto-dismiss)
    pub created_at: std::time::Instant,
    /// Duration to display (in seconds)
    pub duration_secs: u64,
}

impl Notification {
    /// Create a new notification
    pub fn new(message: impl Into<String>, notification_type: NotificationType) -> Self {
        Self {
            message: message.into(),
            notification_type,
            created_at: std::time::Instant::now(),
            duration_secs: 3,
        }
    }

    /// Create an info notification
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Info)
    }

    /// Create a success notification
    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Success)
    }

    /// Create a warning notification
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Warning)
    }

    /// Create an error notification
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Error)
    }

    /// Set the duration for this notification
    pub fn with_duration(mut self, seconds: u64) -> Self {
        self.duration_secs = seconds;
        self
    }

    /// Check if the notification has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs() >= self.duration_secs
    }

    /// Get remaining time as a fraction (0.0 to 1.0)
    pub fn remaining_fraction(&self) -> f64 {
        let elapsed = self.created_at.elapsed().as_secs_f64();
        let total = self.duration_secs as f64;
        (1.0 - elapsed / total).clamp(0.0, 1.0)
    }
}

/// Widget for rendering a notification
pub struct NotificationWidget<'a> {
    notification: &'a Notification,
}

impl<'a> NotificationWidget<'a> {
    /// Create a new notification widget
    pub fn new(notification: &'a Notification) -> Self {
        Self { notification }
    }
}

impl<'a> Widget for NotificationWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let color = self.notification.notification_type.color();
        let icon = self.notification.notification_type.icon();
        let title = self.notification.notification_type.title();

        // Clear the area first
        Clear.render(area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .title(format!(" {} {} ", icon, title))
            .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD));

        let paragraph = Paragraph::new(self.notification.message.as_str())
            .style(Style::default().fg(Color::White))
            .block(block);

        paragraph.render(area, buf);
    }
}

/// A queue of notifications to display
#[derive(Debug, Default)]
pub struct NotificationQueue {
    notifications: Vec<Notification>,
}

impl NotificationQueue {
    /// Create a new notification queue
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a notification to the queue
    pub fn push(&mut self, notification: Notification) {
        self.notifications.push(notification);
    }

    /// Remove expired notifications
    pub fn remove_expired(&mut self) {
        self.notifications.retain(|n| !n.is_expired());
    }

    /// Get the current notification to display (if any)
    pub fn current(&self) -> Option<&Notification> {
        self.notifications.first()
    }

    /// Check if there are any notifications
    pub fn is_empty(&self) -> bool {
        self.notifications.is_empty()
    }

    /// Get the number of notifications
    pub fn len(&self) -> usize {
        self.notifications.len()
    }

    /// Clear all notifications
    pub fn clear(&mut self) {
        self.notifications.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let n = Notification::info("Test message");
        assert_eq!(n.message, "Test message");
        assert_eq!(n.notification_type, NotificationType::Info);
    }

    #[test]
    fn test_notification_types() {
        assert_eq!(NotificationType::Info.color(), Color::Blue);
        assert_eq!(NotificationType::Success.color(), Color::Green);
        assert_eq!(NotificationType::Warning.color(), Color::Yellow);
        assert_eq!(NotificationType::Error.color(), Color::Red);
    }

    #[test]
    fn test_notification_queue() {
        let mut queue = NotificationQueue::new();
        assert!(queue.is_empty());

        queue.push(Notification::info("First"));
        queue.push(Notification::success("Second"));

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.current().unwrap().message, "First");
    }
}
