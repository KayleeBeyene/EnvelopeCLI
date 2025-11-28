//! Error dialog widget
//!
//! Displays detailed error information with recovery suggestions.

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::error::EnvelopeError;

/// An error dialog with details and suggestions
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    /// The error title/summary
    pub title: String,
    /// Detailed error message
    pub details: String,
    /// Suggested recovery actions
    pub suggestions: Vec<String>,
    /// Technical details (for advanced users)
    pub technical: Option<String>,
}

impl ErrorInfo {
    /// Create error info from an EnvelopeError
    pub fn from_error(error: &EnvelopeError) -> Self {
        let (title, details, suggestions, technical) = match error {
            EnvelopeError::Config(msg) => (
                "Configuration Error".to_string(),
                msg.clone(),
                vec![
                    "Check your settings file for syntax errors".to_string(),
                    "Try running 'envelope init' to reset configuration".to_string(),
                ],
                None,
            ),
            EnvelopeError::Io(msg) => (
                "I/O Error".to_string(),
                msg.clone(),
                vec![
                    "Check that you have write permissions to the data directory".to_string(),
                    "Ensure there is enough disk space".to_string(),
                    "Check if the file is locked by another process".to_string(),
                ],
                None,
            ),
            EnvelopeError::Json(msg) => (
                "Data File Error".to_string(),
                format!("Failed to read or write data: {}", msg),
                vec![
                    "The data file may be corrupted".to_string(),
                    "Try restoring from a backup with 'envelope backup restore'".to_string(),
                ],
                Some(msg.clone()),
            ),
            EnvelopeError::Validation(msg) => (
                "Validation Error".to_string(),
                msg.clone(),
                vec!["Review the input values and try again".to_string()],
                None,
            ),
            EnvelopeError::NotFound {
                entity_type,
                identifier,
            } => (
                format!("{} Not Found", entity_type),
                format!(
                    "Could not find {} with identifier '{}'",
                    entity_type.to_lowercase(),
                    identifier
                ),
                vec![
                    format!("Check that the {} exists", entity_type.to_lowercase()),
                    format!(
                        "Use 'envelope {} list' to see available {}s",
                        entity_type.to_lowercase(),
                        entity_type.to_lowercase()
                    ),
                ],
                None,
            ),
            EnvelopeError::Duplicate {
                entity_type,
                identifier,
            } => (
                format!("Duplicate {}", entity_type),
                format!("{} '{}' already exists", entity_type, identifier),
                vec![
                    "Use a different name".to_string(),
                    format!("Edit the existing {} instead", entity_type.to_lowercase()),
                ],
                None,
            ),
            EnvelopeError::Budget(msg) => (
                "Budget Error".to_string(),
                msg.clone(),
                vec![
                    "Review your budget allocations".to_string(),
                    "Check the 'Available to Budget' amount".to_string(),
                ],
                None,
            ),
            EnvelopeError::Reconciliation(msg) => (
                "Reconciliation Error".to_string(),
                msg.clone(),
                vec![
                    "Review the reconciliation difference".to_string(),
                    "Check for missing or duplicate transactions".to_string(),
                ],
                None,
            ),
            EnvelopeError::Import(msg) => (
                "Import Error".to_string(),
                msg.clone(),
                vec![
                    "Check the CSV file format".to_string(),
                    "Ensure the column mapping is correct".to_string(),
                    "Try importing with a different preset".to_string(),
                ],
                None,
            ),
            EnvelopeError::Export(msg) => (
                "Export Error".to_string(),
                msg.clone(),
                vec![
                    "Check that you have write permissions to the output path".to_string(),
                    "Ensure there is enough disk space".to_string(),
                ],
                None,
            ),
            EnvelopeError::Encryption(msg) => (
                "Encryption Error".to_string(),
                msg.clone(),
                vec![
                    "Check that you entered the correct passphrase".to_string(),
                    "If you forgot your passphrase, data cannot be recovered".to_string(),
                ],
                None,
            ),
            EnvelopeError::Locked(msg) => (
                "Transaction Locked".to_string(),
                msg.clone(),
                vec![
                    "Reconciled transactions cannot be edited".to_string(),
                    "Unlock the transaction first with 'envelope transaction unlock'".to_string(),
                ],
                None,
            ),
            EnvelopeError::InsufficientFunds {
                category,
                needed,
                available,
            } => (
                "Insufficient Funds".to_string(),
                format!(
                    "Category '{}' has insufficient funds: need ${:.2}, have ${:.2}",
                    category,
                    *needed as f64 / 100.0,
                    *available as f64 / 100.0
                ),
                vec![
                    "Move funds from another category".to_string(),
                    "Assign more funds to this category".to_string(),
                ],
                None,
            ),
            EnvelopeError::Storage(msg) => (
                "Storage Error".to_string(),
                msg.clone(),
                vec![
                    "Check that the data directory is accessible".to_string(),
                    "Try running with elevated permissions".to_string(),
                ],
                Some(msg.clone()),
            ),
            EnvelopeError::Tui(msg) => (
                "Interface Error".to_string(),
                msg.clone(),
                vec![
                    "Try resizing your terminal window".to_string(),
                    "Use the CLI commands instead".to_string(),
                ],
                None,
            ),
        };

        Self {
            title,
            details,
            suggestions,
            technical,
        }
    }

    /// Create a simple error info
    pub fn simple(title: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            details: details.into(),
            suggestions: vec![],
            technical: None,
        }
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }
}

/// Widget for rendering an error dialog
pub struct ErrorDialog<'a> {
    error: &'a ErrorInfo,
    show_technical: bool,
}

impl<'a> ErrorDialog<'a> {
    /// Create a new error dialog widget
    pub fn new(error: &'a ErrorInfo) -> Self {
        Self {
            error,
            show_technical: false,
        }
    }

    /// Show technical details
    pub fn with_technical(mut self, show: bool) -> Self {
        self.show_technical = show;
        self
    }
}

impl<'a> Widget for ErrorDialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area first
        Clear.render(area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title(format!(" Error: {} ", self.error.title))
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

        let inner = block.inner(area);
        block.render(area, buf);

        // Calculate layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Details
                Constraint::Min(1),    // Suggestions
                Constraint::Length(1), // Close hint
            ])
            .split(inner);

        // Render details
        let details = Paragraph::new(self.error.details.as_str())
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        details.render(chunks[0], buf);

        // Render suggestions
        if !self.error.suggestions.is_empty() {
            let mut lines: Vec<Line> = vec![Line::from(Span::styled(
                "Suggestions:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))];

            for suggestion in &self.error.suggestions {
                lines.push(Line::from(vec![
                    Span::raw("  - "),
                    Span::raw(suggestion.as_str()),
                ]));
            }

            let suggestions = Paragraph::new(lines)
                .style(Style::default().fg(Color::Gray))
                .wrap(Wrap { trim: true });
            suggestions.render(chunks[1], buf);
        }

        // Render close hint
        let close_hint = Paragraph::new("Press Esc or Enter to close")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        close_hint.render(chunks[2], buf);
    }
}

/// Calculate the area for an error dialog (centered in parent)
pub fn error_dialog_area(parent: Rect) -> Rect {
    let width = (parent.width * 70 / 100).clamp(40, 80);
    let height = (parent.height * 50 / 100).clamp(10, 20);

    let x = parent.x + (parent.width - width) / 2;
    let y = parent.y + (parent.height - height) / 2;

    Rect::new(x, y, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_info_from_validation_error() {
        let error = EnvelopeError::Validation("Name cannot be empty".to_string());
        let info = ErrorInfo::from_error(&error);

        assert_eq!(info.title, "Validation Error");
        assert!(info.details.contains("Name cannot be empty"));
    }

    #[test]
    fn test_error_info_from_not_found() {
        let error = EnvelopeError::NotFound {
            entity_type: "Account",
            identifier: "Checking".to_string(),
        };
        let info = ErrorInfo::from_error(&error);

        assert_eq!(info.title, "Account Not Found");
        assert!(info.details.contains("Checking"));
    }

    #[test]
    fn test_simple_error_info() {
        let info =
            ErrorInfo::simple("Test Error", "Something went wrong").with_suggestion("Try again");

        assert_eq!(info.title, "Test Error");
        assert_eq!(info.details, "Something went wrong");
        assert_eq!(info.suggestions.len(), 1);
    }
}
