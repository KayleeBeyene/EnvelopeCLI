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
