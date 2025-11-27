//! Storage initialization
//!
//! Handles first-run setup and default data creation

use crate::config::paths::EnvelopePaths;
use crate::error::EnvelopeError;
use crate::models::{Category, DefaultCategoryGroup};

use super::categories::CategoryData;
use super::file_io::write_json_atomic;

/// Initialize storage for a fresh installation
///
/// Creates default category groups and basic structure
pub fn initialize_storage(paths: &EnvelopePaths) -> Result<(), EnvelopeError> {
    // Ensure all directories exist
    paths.ensure_directories()?;

    // Create default categories if budget.json doesn't exist
    if !paths.budget_file().exists() {
        create_default_categories(paths)?;
    }

    Ok(())
}

/// Create default category groups and some starter categories
fn create_default_categories(paths: &EnvelopePaths) -> Result<(), EnvelopeError> {
    let mut groups = Vec::new();
    let mut categories = Vec::new();

    // Create default groups with predefined categories
    for (i, default_group) in DefaultCategoryGroup::all().iter().enumerate() {
        let group = default_group.to_group(i as i32);
        let group_id = group.id;
        groups.push(group);

        // Add default categories for each group
        let default_cats = match default_group {
            DefaultCategoryGroup::Bills => vec![
                "Rent/Mortgage",
                "Electric",
                "Water",
                "Internet",
                "Phone",
                "Insurance",
            ],
            DefaultCategoryGroup::Needs => vec![
                "Groceries",
                "Transportation",
                "Medical",
                "Household",
            ],
            DefaultCategoryGroup::Wants => vec![
                "Dining Out",
                "Entertainment",
                "Shopping",
                "Subscriptions",
            ],
            DefaultCategoryGroup::Savings => vec![
                "Emergency Fund",
                "Vacation",
                "Large Purchases",
            ],
        };

        for (j, cat_name) in default_cats.into_iter().enumerate() {
            let category = Category::with_sort_order(cat_name, group_id, j as i32);
            categories.push(category);
        }
    }

    let data = CategoryData { groups, categories };
    write_json_atomic(paths.budget_file(), &data)?;

    Ok(())
}

/// Check if storage needs initialization
pub fn needs_initialization(paths: &EnvelopePaths) -> bool {
    !paths.budget_file().exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CategoryGroup;
    use tempfile::TempDir;

    #[test]
    fn test_initialize_storage() {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());

        assert!(needs_initialization(&paths));

        initialize_storage(&paths).unwrap();

        assert!(!needs_initialization(&paths));
        assert!(paths.budget_file().exists());
        assert!(paths.data_dir().exists());
        assert!(paths.backup_dir().exists());
    }

    #[test]
    fn test_default_categories_created() {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());

        initialize_storage(&paths).unwrap();

        // Load and verify
        let content = std::fs::read_to_string(paths.budget_file()).unwrap();
        let data: CategoryData = serde_json::from_str(&content).unwrap();

        // Should have 4 default groups
        assert_eq!(data.groups.len(), 4);

        // Should have categories in each group
        assert!(!data.categories.is_empty());

        // Verify group names
        let group_names: Vec<_> = data.groups.iter().map(|g| g.name.as_str()).collect();
        assert!(group_names.contains(&"Bills"));
        assert!(group_names.contains(&"Needs"));
        assert!(group_names.contains(&"Wants"));
        assert!(group_names.contains(&"Savings"));
    }

    #[test]
    fn test_doesnt_overwrite_existing() {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());

        // First initialization
        initialize_storage(&paths).unwrap();

        // Modify the file
        let custom_data = CategoryData {
            groups: vec![CategoryGroup::new("Custom Group")],
            categories: vec![],
        };
        write_json_atomic(paths.budget_file(), &custom_data).unwrap();

        // Second initialization should not overwrite
        initialize_storage(&paths).unwrap();

        let content = std::fs::read_to_string(paths.budget_file()).unwrap();
        let data: CategoryData = serde_json::from_str(&content).unwrap();

        // Should still have our custom data
        assert_eq!(data.groups.len(), 1);
        assert_eq!(data.groups[0].name, "Custom Group");
    }
}
