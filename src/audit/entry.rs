//! Audit entry data structures
//!
//! Defines the structure of audit log entries including operation types,
//! entity types, and the entry format itself.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types of operations that can be audited
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    /// Entity was created
    Create,
    /// Entity was updated
    Update,
    /// Entity was deleted
    Delete,
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Create => write!(f, "CREATE"),
            Operation::Update => write!(f, "UPDATE"),
            Operation::Delete => write!(f, "DELETE"),
        }
    }
}

/// Types of entities that can be audited
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Account,
    Transaction,
    Category,
    CategoryGroup,
    BudgetAllocation,
    Payee,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Account => write!(f, "Account"),
            EntityType::Transaction => write!(f, "Transaction"),
            EntityType::Category => write!(f, "Category"),
            EntityType::CategoryGroup => write!(f, "CategoryGroup"),
            EntityType::BudgetAllocation => write!(f, "BudgetAllocation"),
            EntityType::Payee => write!(f, "Payee"),
        }
    }
}

/// A single audit log entry
///
/// Records a single operation on an entity with optional before/after values
/// for tracking changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// When the operation occurred (UTC)
    pub timestamp: DateTime<Utc>,

    /// Type of operation performed
    pub operation: Operation,

    /// Type of entity affected
    pub entity_type: EntityType,

    /// ID of the affected entity
    pub entity_id: String,

    /// Human-readable description of the entity (e.g., account name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_name: Option<String>,

    /// JSON representation of the entity before the operation (for updates/deletes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<serde_json::Value>,

    /// JSON representation of the entity after the operation (for creates/updates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<serde_json::Value>,

    /// Human-readable diff summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_summary: Option<String>,
}

impl AuditEntry {
    /// Create a new audit entry for a create operation
    pub fn create<T: Serialize>(
        entity_type: EntityType,
        entity_id: impl Into<String>,
        entity_name: Option<String>,
        entity: &T,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            operation: Operation::Create,
            entity_type,
            entity_id: entity_id.into(),
            entity_name,
            before: None,
            after: serde_json::to_value(entity).ok(),
            diff_summary: None,
        }
    }

    /// Create a new audit entry for an update operation
    pub fn update<T: Serialize>(
        entity_type: EntityType,
        entity_id: impl Into<String>,
        entity_name: Option<String>,
        before: &T,
        after: &T,
        diff_summary: Option<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            operation: Operation::Update,
            entity_type,
            entity_id: entity_id.into(),
            entity_name,
            before: serde_json::to_value(before).ok(),
            after: serde_json::to_value(after).ok(),
            diff_summary,
        }
    }

    /// Create a new audit entry for a delete operation
    pub fn delete<T: Serialize>(
        entity_type: EntityType,
        entity_id: impl Into<String>,
        entity_name: Option<String>,
        entity: &T,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            operation: Operation::Delete,
            entity_type,
            entity_id: entity_id.into(),
            entity_name,
            before: serde_json::to_value(entity).ok(),
            after: None,
            diff_summary: None,
        }
    }

    /// Format the entry for human-readable output
    pub fn format_human_readable(&self) -> String {
        let mut output = format!(
            "[{}] {} {} {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            self.operation,
            self.entity_type,
            self.entity_id
        );

        if let Some(name) = &self.entity_name {
            output.push_str(&format!(" ({})", name));
        }

        if let Some(diff) = &self.diff_summary {
            output.push_str(&format!("\n  Changes: {}", diff));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_operation_display() {
        assert_eq!(Operation::Create.to_string(), "CREATE");
        assert_eq!(Operation::Update.to_string(), "UPDATE");
        assert_eq!(Operation::Delete.to_string(), "DELETE");
    }

    #[test]
    fn test_entity_type_display() {
        assert_eq!(EntityType::Account.to_string(), "Account");
        assert_eq!(EntityType::Transaction.to_string(), "Transaction");
    }

    #[test]
    fn test_create_entry() {
        let data = json!({"name": "Checking", "balance": 1000});
        let entry = AuditEntry::create(
            EntityType::Account,
            "acc-12345678",
            Some("Checking".to_string()),
            &data,
        );

        assert_eq!(entry.operation, Operation::Create);
        assert_eq!(entry.entity_type, EntityType::Account);
        assert_eq!(entry.entity_id, "acc-12345678");
        assert!(entry.before.is_none());
        assert!(entry.after.is_some());
    }

    #[test]
    fn test_update_entry() {
        let before = json!({"name": "Checking", "balance": 1000});
        let after = json!({"name": "Checking", "balance": 1500});

        let entry = AuditEntry::update(
            EntityType::Account,
            "acc-12345678",
            Some("Checking".to_string()),
            &before,
            &after,
            Some("balance: 1000 -> 1500".to_string()),
        );

        assert_eq!(entry.operation, Operation::Update);
        assert!(entry.before.is_some());
        assert!(entry.after.is_some());
        assert_eq!(
            entry.diff_summary,
            Some("balance: 1000 -> 1500".to_string())
        );
    }

    #[test]
    fn test_delete_entry() {
        let data = json!({"name": "Old Account"});
        let entry = AuditEntry::delete(
            EntityType::Account,
            "acc-12345678",
            Some("Old Account".to_string()),
            &data,
        );

        assert_eq!(entry.operation, Operation::Delete);
        assert!(entry.before.is_some());
        assert!(entry.after.is_none());
    }

    #[test]
    fn test_serialization() {
        let data = json!({"name": "Test"});
        let entry = AuditEntry::create(EntityType::Account, "acc-123", None, &data);

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: AuditEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.operation, Operation::Create);
        assert_eq!(deserialized.entity_type, EntityType::Account);
    }

    #[test]
    fn test_human_readable_format() {
        let data = json!({"name": "Checking"});
        let entry = AuditEntry::create(
            EntityType::Account,
            "acc-12345678",
            Some("Checking".to_string()),
            &data,
        );

        let formatted = entry.format_human_readable();
        assert!(formatted.contains("CREATE"));
        assert!(formatted.contains("Account"));
        assert!(formatted.contains("acc-12345678"));
        assert!(formatted.contains("Checking"));
    }
}
