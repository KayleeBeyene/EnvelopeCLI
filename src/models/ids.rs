//! Strongly-typed ID wrappers for all entity types
//!
//! Using newtype wrappers prevents accidentally mixing up IDs from different
//! entity types at compile time.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Macro to generate ID newtype wrappers
macro_rules! define_id {
    ($name:ident, $display_prefix:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Create a new random ID
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Create an ID from an existing UUID
            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Get the underlying UUID
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            /// Parse an ID from a string
            pub fn parse(s: &str) -> Result<Self, uuid::Error> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}{}", $display_prefix, &self.0.to_string()[..8])
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                // Try to parse the full UUID
                if let Ok(uuid) = Uuid::parse_str(s) {
                    return Ok(Self(uuid));
                }
                // Try stripping common prefixes
                let s = s.strip_prefix($display_prefix).unwrap_or(s);
                Ok(Self(Uuid::parse_str(s)?))
            }
        }
    };
}

define_id!(AccountId, "acc-");
define_id!(TransactionId, "txn-");
define_id!(CategoryId, "cat-");
define_id!(CategoryGroupId, "grp-");
define_id!(PayeeId, "pay-");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_id_creation() {
        let id = AccountId::new();
        assert!(!id.as_uuid().is_nil());
    }

    #[test]
    fn test_id_display() {
        let id = AccountId::new();
        let display = format!("{}", id);
        assert!(display.starts_with("acc-"));
        assert_eq!(display.len(), 12); // "acc-" + 8 chars
    }

    #[test]
    fn test_id_equality() {
        let id1 = AccountId::new();
        let id2 = id1;
        assert_eq!(id1, id2);

        let id3 = AccountId::new();
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_id_serialization() {
        let id = AccountId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: AccountId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_id_parse() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = AccountId::parse(uuid_str).unwrap();
        assert_eq!(id.as_uuid().to_string(), uuid_str);
    }

    #[test]
    fn test_different_id_types_not_mixable() {
        // This test documents that different ID types are distinct at compile time
        let account_id = AccountId::new();
        let transaction_id = TransactionId::new();

        // These are different types - can't be compared directly
        // This would fail to compile:
        // assert_ne!(account_id, transaction_id);

        // But we can compare their underlying UUIDs if needed
        assert_ne!(account_id.as_uuid(), transaction_id.as_uuid());
    }
}
