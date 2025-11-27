//! Audit logging system for EnvelopeCLI
//!
//! Records all create, update, delete operations with before/after values
//! in an append-only audit log.
//!
//! # Architecture
//!
//! The audit system consists of three components:
//!
//! - `AuditEntry`: Represents a single audit log entry with timestamp, operation,
//!   entity information, and optional before/after values.
//! - `AuditLogger`: Handles writing entries to the audit log file using a
//!   line-delimited JSON format (JSONL).
//! - `generate_diff`: Utility function to create human-readable diffs between
//!   entity states.
//!
//! # Example
//!
//! ```rust,ignore
//! use envelope::audit::{AuditEntry, AuditLogger, EntityType, generate_diff};
//! use serde_json::json;
//!
//! let logger = AuditLogger::new(audit_log_path);
//!
//! // Log a create operation
//! let entry = AuditEntry::create(
//!     EntityType::Account,
//!     "acc-12345678",
//!     Some("Checking".to_string()),
//!     &account,
//! );
//! logger.log(&entry)?;
//!
//! // Log an update with diff
//! let diff = generate_diff(&before_json, &after_json);
//! let entry = AuditEntry::update(
//!     EntityType::Account,
//!     "acc-12345678",
//!     Some("Checking".to_string()),
//!     &before,
//!     &after,
//!     diff,
//! );
//! logger.log(&entry)?;
//! ```

mod diff;
mod entry;
mod logger;

pub use diff::{generate_detailed_diff, generate_diff};
pub use entry::{AuditEntry, EntityType, Operation};
pub use logger::AuditLogger;
