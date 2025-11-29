//! Command definitions for the command palette
//!
//! Defines all available commands that can be executed via the command palette

/// A command that can be executed
#[derive(Debug, Clone)]
pub struct Command {
    /// Command name (what user types)
    pub name: &'static str,
    /// Short description
    pub description: &'static str,
    /// Keyboard shortcut (if any)
    pub shortcut: Option<&'static str>,
    /// Command action
    pub action: CommandAction,
}

/// Actions that commands can perform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAction {
    // Navigation
    ViewAccounts,
    ViewBudget,
    ViewReports,
    ViewRegister,

    // Account operations
    AddAccount,
    EditAccount,
    ArchiveAccount,

    // Transaction operations
    AddTransaction,
    EditTransaction,
    DeleteTransaction,
    ClearTransaction,

    // Budget operations
    MoveFunds,
    AssignBudget,
    NextPeriod,
    PrevPeriod,

    // Category operations
    AddCategory,
    AddGroup,
    EditCategory,
    DeleteCategory,
    EditGroup,
    DeleteGroup,

    // General
    Help,
    Quit,
    Refresh,
    ToggleArchived,

    // Target operations
    AutoFillTargets,
}

/// All available commands
pub static COMMANDS: &[Command] = &[
    // Navigation commands
    Command {
        name: "accounts",
        description: "View all accounts",
        shortcut: Some("1"),
        action: CommandAction::ViewAccounts,
    },
    Command {
        name: "budget",
        description: "View budget",
        shortcut: Some("2"),
        action: CommandAction::ViewBudget,
    },
    Command {
        name: "reports",
        description: "View reports",
        shortcut: Some("3"),
        action: CommandAction::ViewReports,
    },
    Command {
        name: "register",
        description: "View transactions for selected account",
        shortcut: Some("Enter"),
        action: CommandAction::ViewRegister,
    },
    // Transaction commands
    Command {
        name: "add-transaction",
        description: "Add a new transaction",
        shortcut: Some("a"),
        action: CommandAction::AddTransaction,
    },
    Command {
        name: "edit-transaction",
        description: "Edit selected transaction",
        shortcut: Some("e"),
        action: CommandAction::EditTransaction,
    },
    Command {
        name: "delete-transaction",
        description: "Delete selected transaction",
        shortcut: Some("Ctrl+d"),
        action: CommandAction::DeleteTransaction,
    },
    Command {
        name: "clear-transaction",
        description: "Toggle transaction cleared status",
        shortcut: Some("c"),
        action: CommandAction::ClearTransaction,
    },
    // Budget commands
    Command {
        name: "move-funds",
        description: "Move funds between categories",
        shortcut: Some("m"),
        action: CommandAction::MoveFunds,
    },
    Command {
        name: "assign",
        description: "Assign funds to category",
        shortcut: None,
        action: CommandAction::AssignBudget,
    },
    Command {
        name: "next-period",
        description: "Go to next budget period",
        shortcut: Some("]"),
        action: CommandAction::NextPeriod,
    },
    Command {
        name: "prev-period",
        description: "Go to previous budget period",
        shortcut: Some("["),
        action: CommandAction::PrevPeriod,
    },
    // Account commands
    Command {
        name: "add-account",
        description: "Create a new account",
        shortcut: None,
        action: CommandAction::AddAccount,
    },
    Command {
        name: "edit-account",
        description: "Edit selected account",
        shortcut: None,
        action: CommandAction::EditAccount,
    },
    Command {
        name: "archive-account",
        description: "Archive selected account",
        shortcut: None,
        action: CommandAction::ArchiveAccount,
    },
    Command {
        name: "toggle-archived",
        description: "Show/hide archived accounts",
        shortcut: Some("A"),
        action: CommandAction::ToggleArchived,
    },
    // Category commands
    Command {
        name: "add-category",
        description: "Create a new category",
        shortcut: Some("a"),
        action: CommandAction::AddCategory,
    },
    Command {
        name: "add-group",
        description: "Create a new category group",
        shortcut: Some("A"),
        action: CommandAction::AddGroup,
    },
    Command {
        name: "edit-category",
        description: "Edit selected category",
        shortcut: None,
        action: CommandAction::EditCategory,
    },
    Command {
        name: "delete-category",
        description: "Delete selected category",
        shortcut: None,
        action: CommandAction::DeleteCategory,
    },
    Command {
        name: "edit-group",
        description: "Edit selected category group",
        shortcut: Some("E"),
        action: CommandAction::EditGroup,
    },
    Command {
        name: "delete-group",
        description: "Delete selected category group",
        shortcut: Some("D"),
        action: CommandAction::DeleteGroup,
    },
    // General commands
    Command {
        name: "help",
        description: "Show help",
        shortcut: Some("?"),
        action: CommandAction::Help,
    },
    Command {
        name: "quit",
        description: "Quit application",
        shortcut: Some("q"),
        action: CommandAction::Quit,
    },
    Command {
        name: "refresh",
        description: "Refresh data from disk",
        shortcut: None,
        action: CommandAction::Refresh,
    },
    // Target commands
    Command {
        name: "auto-fill-targets",
        description: "Fill budgets from targets",
        shortcut: None,
        action: CommandAction::AutoFillTargets,
    },
];

/// Find a command by name
pub fn find_command(name: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|cmd| cmd.name == name)
}

/// Filter commands by search query
pub fn filter_commands(query: &str) -> Vec<&'static Command> {
    if query.is_empty() {
        COMMANDS.iter().collect()
    } else {
        let query_lower = query.to_lowercase();
        COMMANDS
            .iter()
            .filter(|cmd| {
                cmd.name.to_lowercase().contains(&query_lower)
                    || cmd.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}
