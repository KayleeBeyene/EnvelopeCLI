//! Category and CategoryGroup models
//!
//! Categories are organized into groups for display and organization.
//! Each category tracks budget allocations and spending.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::ids::{CategoryGroupId, CategoryId};

/// A group of related categories (e.g., "Bills", "Needs", "Wants")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryGroup {
    /// Unique identifier
    pub id: CategoryGroupId,

    /// Group name
    pub name: String,

    /// Sort order for display
    pub sort_order: i32,

    /// Whether this group is hidden (collapsed in UI)
    #[serde(default)]
    pub hidden: bool,

    /// When the group was created
    pub created_at: DateTime<Utc>,

    /// When the group was last modified
    pub updated_at: DateTime<Utc>,
}

impl CategoryGroup {
    /// Create a new category group
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: CategoryGroupId::new(),
            name: name.into(),
            sort_order: 0,
            hidden: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new group with a specific sort order
    pub fn with_sort_order(name: impl Into<String>, sort_order: i32) -> Self {
        let mut group = Self::new(name);
        group.sort_order = sort_order;
        group
    }

    /// Validate the group
    pub fn validate(&self) -> Result<(), CategoryValidationError> {
        if self.name.trim().is_empty() {
            return Err(CategoryValidationError::EmptyName);
        }

        if self.name.len() > 50 {
            return Err(CategoryValidationError::NameTooLong(self.name.len()));
        }

        Ok(())
    }
}

impl fmt::Display for CategoryGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// A budget category within a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Unique identifier
    pub id: CategoryId,

    /// Category name
    pub name: String,

    /// The group this category belongs to
    pub group_id: CategoryGroupId,

    /// Sort order within the group
    pub sort_order: i32,

    /// Whether this category is hidden
    #[serde(default)]
    pub hidden: bool,

    /// Goal amount per period (optional)
    pub goal_amount: Option<i64>,

    /// Notes about this category
    #[serde(default)]
    pub notes: String,

    /// When the category was created
    pub created_at: DateTime<Utc>,

    /// When the category was last modified
    pub updated_at: DateTime<Utc>,
}

impl Category {
    /// Create a new category
    pub fn new(name: impl Into<String>, group_id: CategoryGroupId) -> Self {
        let now = Utc::now();
        Self {
            id: CategoryId::new(),
            name: name.into(),
            group_id,
            sort_order: 0,
            hidden: false,
            goal_amount: None,
            notes: String::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new category with a specific sort order
    pub fn with_sort_order(
        name: impl Into<String>,
        group_id: CategoryGroupId,
        sort_order: i32,
    ) -> Self {
        let mut category = Self::new(name, group_id);
        category.sort_order = sort_order;
        category
    }

    /// Set a goal amount
    pub fn set_goal(&mut self, amount: i64) {
        self.goal_amount = Some(amount);
        self.updated_at = Utc::now();
    }

    /// Clear the goal
    pub fn clear_goal(&mut self) {
        self.goal_amount = None;
        self.updated_at = Utc::now();
    }

    /// Move to a different group
    pub fn move_to_group(&mut self, group_id: CategoryGroupId) {
        self.group_id = group_id;
        self.updated_at = Utc::now();
    }

    /// Validate the category
    pub fn validate(&self) -> Result<(), CategoryValidationError> {
        if self.name.trim().is_empty() {
            return Err(CategoryValidationError::EmptyName);
        }

        if self.name.len() > 50 {
            return Err(CategoryValidationError::NameTooLong(self.name.len()));
        }

        if let Some(goal) = self.goal_amount {
            if goal < 0 {
                return Err(CategoryValidationError::NegativeGoal);
            }
        }

        Ok(())
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Default category groups for new budgets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultCategoryGroup {
    Bills,
    Needs,
    Wants,
    Savings,
}

impl DefaultCategoryGroup {
    /// Get all default groups in order
    pub fn all() -> &'static [Self] {
        &[Self::Bills, Self::Needs, Self::Wants, Self::Savings]
    }

    /// Get the name for this default group
    pub fn name(&self) -> &'static str {
        match self {
            Self::Bills => "Bills",
            Self::Needs => "Needs",
            Self::Wants => "Wants",
            Self::Savings => "Savings",
        }
    }

    /// Create a CategoryGroup from this default
    pub fn to_group(&self, sort_order: i32) -> CategoryGroup {
        CategoryGroup::with_sort_order(self.name(), sort_order)
    }
}

/// Validation errors for categories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CategoryValidationError {
    EmptyName,
    NameTooLong(usize),
    NegativeGoal,
}

impl fmt::Display for CategoryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(f, "Category name cannot be empty"),
            Self::NameTooLong(len) => {
                write!(f, "Category name too long ({} chars, max 50)", len)
            }
            Self::NegativeGoal => write!(f, "Goal amount cannot be negative"),
        }
    }
}

impl std::error::Error for CategoryValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_group() {
        let group = CategoryGroup::new("Bills");
        assert_eq!(group.name, "Bills");
        assert_eq!(group.sort_order, 0);
        assert!(!group.hidden);
    }

    #[test]
    fn test_new_category() {
        let group = CategoryGroup::new("Bills");
        let category = Category::new("Rent", group.id);

        assert_eq!(category.name, "Rent");
        assert_eq!(category.group_id, group.id);
        assert!(!category.hidden);
        assert!(category.goal_amount.is_none());
    }

    #[test]
    fn test_category_goal() {
        let group = CategoryGroup::new("Savings");
        let mut category = Category::new("Emergency Fund", group.id);

        category.set_goal(100000); // $1000.00
        assert_eq!(category.goal_amount, Some(100000));

        category.clear_goal();
        assert!(category.goal_amount.is_none());
    }

    #[test]
    fn test_group_validation() {
        let mut group = CategoryGroup::new("Valid");
        assert!(group.validate().is_ok());

        group.name = String::new();
        assert_eq!(group.validate(), Err(CategoryValidationError::EmptyName));

        group.name = "a".repeat(51);
        assert!(matches!(
            group.validate(),
            Err(CategoryValidationError::NameTooLong(_))
        ));
    }

    #[test]
    fn test_category_validation() {
        let group = CategoryGroup::new("Test");
        let mut category = Category::new("Valid", group.id);
        assert!(category.validate().is_ok());

        category.name = String::new();
        assert_eq!(
            category.validate(),
            Err(CategoryValidationError::EmptyName)
        );

        category.name = "Valid".to_string();
        category.goal_amount = Some(-100);
        assert_eq!(
            category.validate(),
            Err(CategoryValidationError::NegativeGoal)
        );
    }

    #[test]
    fn test_default_groups() {
        let defaults = DefaultCategoryGroup::all();
        assert_eq!(defaults.len(), 4);
        assert_eq!(defaults[0].name(), "Bills");
        assert_eq!(defaults[1].name(), "Needs");
    }

    #[test]
    fn test_move_category() {
        let group1 = CategoryGroup::new("Group 1");
        let group2 = CategoryGroup::new("Group 2");
        let mut category = Category::new("Test", group1.id);

        assert_eq!(category.group_id, group1.id);

        category.move_to_group(group2.id);
        assert_eq!(category.group_id, group2.id);
    }

    #[test]
    fn test_serialization() {
        let group = CategoryGroup::new("Test Group");
        let json = serde_json::to_string(&group).unwrap();
        let deserialized: CategoryGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(group.id, deserialized.id);
        assert_eq!(group.name, deserialized.name);

        let category = Category::new("Test Category", group.id);
        let json = serde_json::to_string(&category).unwrap();
        let deserialized: Category = serde_json::from_str(&json).unwrap();
        assert_eq!(category.id, deserialized.id);
        assert_eq!(category.name, deserialized.name);
    }
}
