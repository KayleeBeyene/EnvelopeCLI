//! Custom error types for EnvelopeCLI
//!
//! This module defines the error hierarchy for the application using thiserror
//! for ergonomic error definitions.

use thiserror::Error;

/// The main error type for EnvelopeCLI operations
#[derive(Error, Debug)]
pub enum EnvelopeError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// File I/O errors
    #[error("I/O error: {0}")]
    Io(String),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(String),

    /// Validation errors for data models
    #[error("Validation error: {0}")]
    Validation(String),

    /// Entity not found errors
    #[error("{entity_type} not found: {identifier}")]
    NotFound {
        entity_type: &'static str,
        identifier: String,
    },

    /// Duplicate entity errors
    #[error("{entity_type} already exists: {identifier}")]
    Duplicate {
        entity_type: &'static str,
        identifier: String,
    },

    /// Budget-related errors
    #[error("Budget error: {0}")]
    Budget(String),

    /// Reconciliation errors
    #[error("Reconciliation error: {0}")]
    Reconciliation(String),

    /// Import errors
    #[error("Import error: {0}")]
    Import(String),

    /// Export errors
    #[error("Export error: {0}")]
    Export(String),

    /// Encryption errors
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Transaction is locked (reconciled)
    #[error("Transaction is locked: {0}")]
    Locked(String),

    /// Insufficient funds
    #[error("Insufficient funds in category '{category}': need {needed}, have {available}")]
    InsufficientFunds {
        category: String,
        needed: i64,
        available: i64,
    },

    /// Storage errors
    #[error("Storage error: {0}")]
    Storage(String),

    /// TUI errors
    #[error("TUI error: {0}")]
    Tui(String),
}

impl EnvelopeError {
    /// Create a "not found" error for accounts
    pub fn account_not_found(identifier: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: "Account",
            identifier: identifier.into(),
        }
    }

    /// Create a "not found" error for categories
    pub fn category_not_found(identifier: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: "Category",
            identifier: identifier.into(),
        }
    }

    /// Create a "not found" error for transactions
    pub fn transaction_not_found(identifier: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: "Transaction",
            identifier: identifier.into(),
        }
    }

    /// Create a "not found" error for payees
    pub fn payee_not_found(identifier: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: "Payee",
            identifier: identifier.into(),
        }
    }

    /// Check if this is a "not found" error
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    /// Check if this is a validation error
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Validation(_))
    }

    /// Check if this is a recoverable error (can retry)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Io(_) | Self::Storage(_) | Self::Validation(_) | Self::Encryption(_)
        )
    }

    /// Check if this is a fatal error (should exit)
    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::Config(_))
    }

    /// Get a user-friendly message for this error
    pub fn user_message(&self) -> String {
        match self {
            Self::Config(msg) => format!("Configuration problem: {}", msg),
            Self::Io(msg) => format!("Could not access file: {}", msg),
            Self::Json(msg) => format!("Data file is corrupted: {}", msg),
            Self::Validation(msg) => msg.clone(),
            Self::NotFound { entity_type, identifier } => {
                format!("{} '{}' was not found", entity_type, identifier)
            }
            Self::Duplicate { entity_type, identifier } => {
                format!("{} '{}' already exists", entity_type, identifier)
            }
            Self::Budget(msg) => msg.clone(),
            Self::Reconciliation(msg) => msg.clone(),
            Self::Import(msg) => format!("Import failed: {}", msg),
            Self::Export(msg) => format!("Export failed: {}", msg),
            Self::Encryption(msg) => format!("Encryption error: {}", msg),
            Self::Locked(msg) => format!("Cannot modify locked transaction: {}", msg),
            Self::InsufficientFunds { category, needed, available } => {
                format!(
                    "'{}' doesn't have enough funds (need ${:.2}, have ${:.2})",
                    category,
                    *needed as f64 / 100.0,
                    *available as f64 / 100.0
                )
            }
            Self::Storage(msg) => format!("Storage error: {}", msg),
            Self::Tui(msg) => format!("Display error: {}", msg),
        }
    }

    /// Get recovery suggestions for this error
    pub fn recovery_suggestions(&self) -> Vec<&'static str> {
        match self {
            Self::Config(_) => vec![
                "Check ~/.envelope/config.json for syntax errors",
                "Run 'envelope init' to reset configuration",
            ],
            Self::Io(_) => vec![
                "Check file permissions",
                "Ensure the disk has free space",
                "Try closing other programs that might be using the files",
            ],
            Self::Json(_) => vec![
                "The data file may be corrupted",
                "Restore from backup: 'envelope backup restore'",
            ],
            Self::Validation(_) => vec!["Check your input and try again"],
            Self::NotFound { entity_type, .. } => {
                match *entity_type {
                    "Account" => vec!["Run 'envelope account list' to see available accounts"],
                    "Category" => vec!["Run 'envelope category list' to see available categories"],
                    "Transaction" => vec!["Check the transaction ID and try again"],
                    _ => vec!["Check that the item exists"],
                }
            }
            Self::Duplicate { .. } => vec!["Use a different name", "Edit the existing item instead"],
            Self::Budget(_) => vec!["Check your budget allocations", "Review 'Available to Budget'"],
            Self::Reconciliation(_) => vec![
                "Review the reconciliation difference",
                "Check for missing transactions",
            ],
            Self::Import(_) => vec![
                "Check the CSV file format",
                "Ensure column mapping is correct",
            ],
            Self::Export(_) => vec![
                "Check write permissions to the output path",
                "Ensure there is enough disk space",
            ],
            Self::Encryption(_) => vec![
                "Verify your passphrase",
                "Note: There is no password recovery",
            ],
            Self::Locked(_) => vec![
                "Use 'envelope transaction unlock' to edit",
                "This will require confirmation",
            ],
            Self::InsufficientFunds { .. } => vec![
                "Move funds from another category",
                "Assign more funds to this category",
            ],
            Self::Storage(_) => vec![
                "Check the data directory is accessible",
                "Try with elevated permissions",
            ],
            Self::Tui(_) => vec!["Try resizing your terminal", "Use CLI commands instead"],
        }
    }

    /// Get the exit code for this error
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config(_) => 1,
            Self::Io(_) => 2,
            Self::Json(_) => 3,
            Self::Validation(_) => 4,
            Self::NotFound { .. } => 5,
            Self::Duplicate { .. } => 6,
            Self::Budget(_) => 7,
            Self::Reconciliation(_) => 8,
            Self::Import(_) => 9,
            Self::Export(_) => 10,
            Self::Encryption(_) => 11,
            Self::Locked(_) => 12,
            Self::InsufficientFunds { .. } => 13,
            Self::Storage(_) => 14,
            Self::Tui(_) => 15,
        }
    }
}

/// Format an error for CLI output with suggestions
pub fn format_cli_error(error: &EnvelopeError) -> String {
    let mut output = format!("Error: {}\n", error.user_message());

    let suggestions = error.recovery_suggestions();
    if !suggestions.is_empty() {
        output.push_str("\nSuggestions:\n");
        for suggestion in suggestions {
            output.push_str(&format!("  - {}\n", suggestion));
        }
    }

    output
}

// Implement From traits for common error types

impl From<std::io::Error> for EnvelopeError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for EnvelopeError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

/// Result type alias for EnvelopeCLI operations
pub type EnvelopeResult<T> = Result<T, EnvelopeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EnvelopeError::Config("test error".into());
        assert_eq!(err.to_string(), "Configuration error: test error");
    }

    #[test]
    fn test_not_found_error() {
        let err = EnvelopeError::account_not_found("Checking");
        assert_eq!(err.to_string(), "Account not found: Checking");
        assert!(err.is_not_found());
    }

    #[test]
    fn test_insufficient_funds_error() {
        let err = EnvelopeError::InsufficientFunds {
            category: "Groceries".into(),
            needed: 5000,
            available: 3000,
        };
        assert_eq!(
            err.to_string(),
            "Insufficient funds in category 'Groceries': need 5000, have 3000"
        );
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let envelope_err: EnvelopeError = io_err.into();
        assert!(matches!(envelope_err, EnvelopeError::Io(_)));
    }
}
