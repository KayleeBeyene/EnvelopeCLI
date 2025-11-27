//! Category CLI commands
//!
//! Implements CLI commands for category and category group management.

use clap::Subcommand;

use crate::display::category::{
    format_category_details, format_category_tree, format_group_details, format_group_list,
};
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::services::CategoryService;
use crate::storage::Storage;

/// Category subcommands
#[derive(Subcommand)]
pub enum CategoryCommands {
    /// List all categories (organized by group)
    List,

    /// Create a new category
    Create {
        /// Category name
        name: String,
        /// Category group name or ID
        #[arg(short, long)]
        group: String,
        /// Goal amount (e.g., "500" or "500.00")
        #[arg(long)]
        goal: Option<String>,
    },

    /// Show category details
    Show {
        /// Category name or ID
        category: String,
    },

    /// Edit a category
    Edit {
        /// Category name or ID
        category: String,
        /// New name
        #[arg(short, long)]
        name: Option<String>,
        /// New goal amount
        #[arg(long)]
        goal: Option<String>,
        /// Clear the goal
        #[arg(long)]
        clear_goal: bool,
    },

    /// Move a category to a different group
    Move {
        /// Category name or ID
        category: String,
        /// Target group name or ID
        #[arg(short, long)]
        to: String,
    },

    /// Delete a category
    Delete {
        /// Category name or ID
        category: String,
    },

    /// Create a new category group
    #[command(name = "create-group")]
    CreateGroup {
        /// Group name
        name: String,
    },

    /// List all category groups
    #[command(name = "list-groups")]
    ListGroups,

    /// Show group details
    #[command(name = "show-group")]
    ShowGroup {
        /// Group name or ID
        group: String,
    },

    /// Edit a category group
    #[command(name = "edit-group")]
    EditGroup {
        /// Group name or ID
        group: String,
        /// New name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Delete a category group
    #[command(name = "delete-group")]
    DeleteGroup {
        /// Group name or ID
        group: String,
        /// Force delete (also deletes all categories in the group)
        #[arg(long)]
        force: bool,
    },
}

/// Handle a category command
pub fn handle_category_command(storage: &Storage, cmd: CategoryCommands) -> EnvelopeResult<()> {
    let service = CategoryService::new(storage);

    match cmd {
        CategoryCommands::List => {
            let groups = service.list_groups_with_categories()?;
            print!("{}", format_category_tree(&groups));
        }

        CategoryCommands::Create { name, group, goal } => {
            let group = service.find_group(&group)?.ok_or_else(|| EnvelopeError::NotFound {
                entity_type: "Category Group",
                identifier: group.clone(),
            })?;

            let category = service.create_category(&name, group.id)?;

            // Set goal if provided
            if let Some(goal_str) = goal {
                let goal_money = crate::models::Money::parse(&goal_str).map_err(|e| {
                    EnvelopeError::Validation(format!("Invalid goal amount: {}", e))
                })?;
                service.update_category(category.id, None, Some(goal_money.cents()), false)?;
            }

            println!("Created category: {}", category.name);
            println!("  Group: {}", group.name);
            println!("  ID: {}", category.id);
        }

        CategoryCommands::Show { category } => {
            let cat = service.find_category(&category)?.ok_or_else(|| {
                EnvelopeError::category_not_found(&category)
            })?;

            let group = service.get_group(cat.group_id)?;
            print!("{}", format_category_details(&cat, group.as_ref()));
        }

        CategoryCommands::Edit {
            category,
            name,
            goal,
            clear_goal,
        } => {
            let cat = service.find_category(&category)?.ok_or_else(|| {
                EnvelopeError::category_not_found(&category)
            })?;

            if name.is_none() && goal.is_none() && !clear_goal {
                println!("No changes specified. Use --name, --goal, or --clear-goal.");
                return Ok(());
            }

            let goal_cents = if let Some(goal_str) = goal {
                let goal_money = crate::models::Money::parse(&goal_str).map_err(|e| {
                    EnvelopeError::Validation(format!("Invalid goal amount: {}", e))
                })?;
                Some(goal_money.cents())
            } else {
                None
            };

            let updated = service.update_category(cat.id, name.as_deref(), goal_cents, clear_goal)?;
            println!("Updated category: {}", updated.name);
        }

        CategoryCommands::Move { category, to } => {
            let cat = service.find_category(&category)?.ok_or_else(|| {
                EnvelopeError::category_not_found(&category)
            })?;

            let target_group = service.find_group(&to)?.ok_or_else(|| EnvelopeError::NotFound {
                entity_type: "Category Group",
                identifier: to.clone(),
            })?;

            let moved = service.move_category(cat.id, target_group.id)?;
            println!("Moved '{}' to group '{}'", moved.name, target_group.name);
        }

        CategoryCommands::Delete { category } => {
            let cat = service.find_category(&category)?.ok_or_else(|| {
                EnvelopeError::category_not_found(&category)
            })?;

            service.delete_category(cat.id)?;
            println!("Deleted category: {}", cat.name);
        }

        CategoryCommands::CreateGroup { name } => {
            let group = service.create_group(&name)?;
            println!("Created category group: {}", group.name);
            println!("  ID: {}", group.id);
        }

        CategoryCommands::ListGroups => {
            let groups = service.list_groups()?;
            print!("{}", format_group_list(&groups));
        }

        CategoryCommands::ShowGroup { group } => {
            let g = service.find_group(&group)?.ok_or_else(|| EnvelopeError::NotFound {
                entity_type: "Category Group",
                identifier: group.clone(),
            })?;

            let categories = service.list_categories_in_group(g.id)?;
            print!("{}", format_group_details(&g, &categories));
        }

        CategoryCommands::EditGroup { group, name } => {
            let g = service.find_group(&group)?.ok_or_else(|| EnvelopeError::NotFound {
                entity_type: "Category Group",
                identifier: group.clone(),
            })?;

            if name.is_none() {
                println!("No changes specified. Use --name to change the group name.");
                return Ok(());
            }

            let updated = service.update_group(g.id, name.as_deref())?;
            println!("Updated category group: {}", updated.name);
        }

        CategoryCommands::DeleteGroup { group, force } => {
            let g = service.find_group(&group)?.ok_or_else(|| EnvelopeError::NotFound {
                entity_type: "Category Group",
                identifier: group.clone(),
            })?;

            service.delete_group(g.id, force)?;
            println!("Deleted category group: {}", g.name);
        }
    }

    Ok(())
}
