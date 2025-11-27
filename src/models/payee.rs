//! Payee model
//!
//! Tracks payees and their auto-categorization rules based on historical patterns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use super::ids::{CategoryId, PayeeId};

/// A payee with auto-categorization rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payee {
    /// Unique identifier
    pub id: PayeeId,

    /// Payee name
    pub name: String,

    /// Default category for new transactions with this payee
    pub default_category_id: Option<CategoryId>,

    /// Category usage frequency for learning (category_id -> count)
    #[serde(default)]
    pub category_frequency: HashMap<CategoryId, u32>,

    /// Whether this payee was manually created vs auto-created from transaction
    #[serde(default)]
    pub manual: bool,

    /// When the payee was created
    pub created_at: DateTime<Utc>,

    /// When the payee was last modified
    pub updated_at: DateTime<Utc>,
}

impl Payee {
    /// Create a new payee
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: PayeeId::new(),
            name: name.into(),
            default_category_id: None,
            category_frequency: HashMap::new(),
            manual: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a manually-created payee with a default category
    pub fn with_default_category(name: impl Into<String>, category_id: CategoryId) -> Self {
        let mut payee = Self::new(name);
        payee.default_category_id = Some(category_id);
        payee.manual = true;
        payee
    }

    /// Record a category usage for learning
    pub fn record_category_usage(&mut self, category_id: CategoryId) {
        *self.category_frequency.entry(category_id).or_insert(0) += 1;
        self.updated_at = Utc::now();

        // Auto-update default category if not manually set
        if !self.manual {
            self.update_default_from_frequency();
        }
    }

    /// Update the default category based on frequency
    fn update_default_from_frequency(&mut self) {
        if let Some((&most_used_category, _)) = self
            .category_frequency
            .iter()
            .max_by_key(|(_, count)| *count)
        {
            self.default_category_id = Some(most_used_category);
        }
    }

    /// Get the suggested category (default or most frequent)
    pub fn suggested_category(&self) -> Option<CategoryId> {
        self.default_category_id.or_else(|| {
            self.category_frequency
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(&category_id, _)| category_id)
        })
    }

    /// Set the default category manually
    pub fn set_default_category(&mut self, category_id: CategoryId) {
        self.default_category_id = Some(category_id);
        self.manual = true;
        self.updated_at = Utc::now();
    }

    /// Clear the default category
    pub fn clear_default_category(&mut self) {
        self.default_category_id = None;
        self.manual = false;
        self.updated_at = Utc::now();
    }

    /// Validate the payee
    pub fn validate(&self) -> Result<(), PayeeValidationError> {
        if self.name.trim().is_empty() {
            return Err(PayeeValidationError::EmptyName);
        }

        if self.name.len() > 100 {
            return Err(PayeeValidationError::NameTooLong(self.name.len()));
        }

        Ok(())
    }

    /// Normalize a payee name for matching
    pub fn normalize_name(name: &str) -> String {
        name.trim().to_lowercase()
    }

    /// Check if this payee matches a name (case-insensitive)
    pub fn matches_name(&self, name: &str) -> bool {
        Self::normalize_name(&self.name) == Self::normalize_name(name)
    }

    /// Calculate similarity score for fuzzy matching (0.0 to 1.0)
    pub fn similarity_score(&self, query: &str) -> f64 {
        let name = Self::normalize_name(&self.name);
        let query = Self::normalize_name(query);

        if name == query {
            return 1.0;
        }

        if name.contains(&query) || query.contains(&name) {
            return 0.8;
        }

        // Simple character overlap similarity
        let name_chars: std::collections::HashSet<char> = name.chars().collect();
        let query_chars: std::collections::HashSet<char> = query.chars().collect();
        let intersection = name_chars.intersection(&query_chars).count();
        let union = name_chars.union(&query_chars).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

impl fmt::Display for Payee {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Validation errors for payees
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayeeValidationError {
    EmptyName,
    NameTooLong(usize),
}

impl fmt::Display for PayeeValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(f, "Payee name cannot be empty"),
            Self::NameTooLong(len) => {
                write!(f, "Payee name too long ({} chars, max 100)", len)
            }
        }
    }
}

impl std::error::Error for PayeeValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_category_id() -> CategoryId {
        CategoryId::new()
    }

    #[test]
    fn test_new_payee() {
        let payee = Payee::new("Test Store");
        assert_eq!(payee.name, "Test Store");
        assert!(payee.default_category_id.is_none());
        assert!(!payee.manual);
    }

    #[test]
    fn test_with_default_category() {
        let category_id = test_category_id();
        let payee = Payee::with_default_category("Test Store", category_id);

        assert_eq!(payee.default_category_id, Some(category_id));
        assert!(payee.manual);
    }

    #[test]
    fn test_category_learning() {
        let mut payee = Payee::new("Grocery Store");
        let groceries = test_category_id();
        let household = test_category_id();

        // Record some usage
        payee.record_category_usage(groceries);
        payee.record_category_usage(groceries);
        payee.record_category_usage(household);

        assert_eq!(payee.category_frequency.get(&groceries), Some(&2));
        assert_eq!(payee.category_frequency.get(&household), Some(&1));

        // Groceries should be the suggested category
        assert_eq!(payee.suggested_category(), Some(groceries));
    }

    #[test]
    fn test_manual_override() {
        let mut payee = Payee::new("Store");
        let learned_category = test_category_id();
        let manual_category = test_category_id();

        // Learn a category
        payee.record_category_usage(learned_category);
        payee.record_category_usage(learned_category);
        assert_eq!(payee.suggested_category(), Some(learned_category));

        // Manual override
        payee.set_default_category(manual_category);
        assert_eq!(payee.suggested_category(), Some(manual_category));
        assert!(payee.manual);

        // Further learning should not change the manual default
        payee.record_category_usage(learned_category);
        assert_eq!(payee.suggested_category(), Some(manual_category));
    }

    #[test]
    fn test_name_matching() {
        let payee = Payee::new("Test Store");
        assert!(payee.matches_name("Test Store"));
        assert!(payee.matches_name("TEST STORE"));
        assert!(payee.matches_name("test store"));
        assert!(!payee.matches_name("Other Store"));
    }

    #[test]
    fn test_similarity_score() {
        let payee = Payee::new("Grocery Store");

        assert_eq!(payee.similarity_score("Grocery Store"), 1.0);
        assert_eq!(payee.similarity_score("grocery store"), 1.0);
        assert!(payee.similarity_score("Grocery") >= 0.8);
        assert!(payee.similarity_score("Store") >= 0.8);
        assert!(payee.similarity_score("XYZ") < 0.5);
    }

    #[test]
    fn test_validation() {
        let mut payee = Payee::new("Valid Name");
        assert!(payee.validate().is_ok());

        payee.name = String::new();
        assert_eq!(payee.validate(), Err(PayeeValidationError::EmptyName));

        payee.name = "a".repeat(101);
        assert!(matches!(
            payee.validate(),
            Err(PayeeValidationError::NameTooLong(_))
        ));
    }

    #[test]
    fn test_serialization() {
        let mut payee = Payee::new("Test Store");
        let category = test_category_id();
        payee.record_category_usage(category);

        let json = serde_json::to_string(&payee).unwrap();
        let deserialized: Payee = serde_json::from_str(&json).unwrap();

        assert_eq!(payee.id, deserialized.id);
        assert_eq!(payee.name, deserialized.name);
        assert_eq!(
            payee.category_frequency.get(&category),
            deserialized.category_frequency.get(&category)
        );
    }
}
