//! Category display formatting
//!
//! Formats categories and groups for terminal output in tree and table views.

use crate::models::{Category, CategoryGroup};
use crate::services::category::CategoryGroupWithCategories;

/// Format categories as a tree structure grouped by category group
pub fn format_category_tree(groups_with_categories: &[CategoryGroupWithCategories]) -> String {
    if groups_with_categories.is_empty() {
        return "No categories found.\n\nRun 'envelope init' to create default categories."
            .to_string();
    }

    let mut output = String::new();

    for (i, gwc) in groups_with_categories.iter().enumerate() {
        // Group header
        output.push_str(&format!("{}\n", gwc.group.name));

        // Categories in group
        if gwc.categories.is_empty() {
            output.push_str("  (no categories)\n");
        } else {
            for (j, category) in gwc.categories.iter().enumerate() {
                let is_last = j == gwc.categories.len() - 1;
                let prefix = if is_last { "└── " } else { "├── " };

                let goal_str = if let Some(goal) = category.goal_amount {
                    format!(" (goal: {})", crate::models::Money::from_cents(goal))
                } else {
                    String::new()
                };

                output.push_str(&format!("  {}{}{}\n", prefix, category.name, goal_str));
            }
        }

        // Add blank line between groups (except after last)
        if i < groups_with_categories.len() - 1 {
            output.push('\n');
        }
    }

    output
}

/// Format a simple list of groups
pub fn format_group_list(groups: &[CategoryGroup]) -> String {
    if groups.is_empty() {
        return "No category groups found.".to_string();
    }

    let mut output = String::new();
    output.push_str("Category Groups:\n");

    for group in groups {
        let hidden = if group.hidden { " (hidden)" } else { "" };
        output.push_str(&format!(
            "  {} - order: {}{}\n",
            group.name, group.sort_order, hidden
        ));
    }

    output
}

/// Format a simple list of categories
pub fn format_category_list(categories: &[Category]) -> String {
    if categories.is_empty() {
        return "No categories found.".to_string();
    }

    let name_width = categories
        .iter()
        .map(|c| c.name.len())
        .max()
        .unwrap_or(4)
        .max(4);

    let mut output = String::new();
    output.push_str(&format!(
        "{:<width$}  {:>10}  {}\n",
        "Category",
        "Goal",
        "ID",
        width = name_width
    ));
    output.push_str(&format!(
        "{:-<width$}  {:->10}  {:-<12}\n",
        "",
        "",
        "",
        width = name_width
    ));

    for category in categories {
        let goal_str = category
            .goal_amount
            .map(|g| crate::models::Money::from_cents(g).to_string())
            .unwrap_or_else(|| "-".to_string());

        output.push_str(&format!(
            "{:<width$}  {:>10}  {}\n",
            category.name,
            goal_str,
            category.id,
            width = name_width
        ));
    }

    output
}

/// Format category details
pub fn format_category_details(category: &Category, group: Option<&CategoryGroup>) -> String {
    let mut output = String::new();

    output.push_str(&format!("Category: {}\n", category.name));
    output.push_str(&format!("  ID:         {}\n", category.id));

    if let Some(g) = group {
        output.push_str(&format!("  Group:      {}\n", g.name));
    }

    output.push_str(&format!(
        "  Hidden:     {}\n",
        if category.hidden { "Yes" } else { "No" }
    ));
    output.push_str(&format!("  Sort Order: {}\n", category.sort_order));

    if let Some(goal) = category.goal_amount {
        output.push_str(&format!(
            "  Goal:       {}\n",
            crate::models::Money::from_cents(goal)
        ));
    }

    if !category.notes.is_empty() {
        output.push_str(&format!("  Notes:      {}\n", category.notes));
    }

    output.push('\n');
    output.push_str(&format!(
        "  Created:  {}\n",
        category.created_at.format("%Y-%m-%d %H:%M UTC")
    ));
    output.push_str(&format!(
        "  Modified: {}\n",
        category.updated_at.format("%Y-%m-%d %H:%M UTC")
    ));

    output
}

/// Format group details
pub fn format_group_details(group: &CategoryGroup, categories: &[Category]) -> String {
    let mut output = String::new();

    output.push_str(&format!("Category Group: {}\n", group.name));
    output.push_str(&format!("  ID:         {}\n", group.id));
    output.push_str(&format!(
        "  Hidden:     {}\n",
        if group.hidden { "Yes" } else { "No" }
    ));
    output.push_str(&format!("  Sort Order: {}\n", group.sort_order));
    output.push_str(&format!("  Categories: {}\n", categories.len()));

    if !categories.is_empty() {
        output.push_str("\n  Categories in this group:\n");
        for category in categories {
            output.push_str(&format!("    - {}\n", category.name));
        }
    }

    output.push('\n');
    output.push_str(&format!(
        "  Created:  {}\n",
        group.created_at.format("%Y-%m-%d %H:%M UTC")
    ));
    output.push_str(&format!(
        "  Modified: {}\n",
        group.updated_at.format("%Y-%m-%d %H:%M UTC")
    ));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_empty_tree() {
        let output = format_category_tree(&[]);
        assert!(output.contains("No categories found"));
    }

    #[test]
    fn test_format_category_tree() {
        let group = CategoryGroup::new("Bills");
        let cat1 = Category::new("Rent", group.id);
        let cat2 = Category::new("Electric", group.id);

        let gwc = CategoryGroupWithCategories {
            group,
            categories: vec![cat1, cat2],
        };

        let output = format_category_tree(&[gwc]);
        assert!(output.contains("Bills"));
        assert!(output.contains("Rent"));
        assert!(output.contains("Electric"));
        assert!(output.contains("├──"));
        assert!(output.contains("└──"));
    }

    #[test]
    fn test_format_category_with_goal() {
        let group = CategoryGroup::new("Savings");
        let mut cat = Category::new("Emergency Fund", group.id);
        cat.set_goal(100000); // $1000

        let gwc = CategoryGroupWithCategories {
            group,
            categories: vec![cat],
        };

        let output = format_category_tree(&[gwc]);
        assert!(output.contains("Emergency Fund"));
        assert!(output.contains("goal:"));
        assert!(output.contains("$1000.00"));
    }
}
