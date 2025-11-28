//! Keybinding definitions
//!
//! Defines all keyboard shortcuts for different contexts

use crossterm::event::{KeyCode, KeyModifiers};

/// A keybinding definition
#[derive(Debug, Clone)]
pub struct Keybinding {
    /// The key code
    pub key: KeyCode,
    /// Required modifiers
    pub modifiers: KeyModifiers,
    /// Description of what the key does
    pub description: &'static str,
    /// Context where this keybinding is active
    pub context: KeyContext,
}

/// Context in which a keybinding is active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyContext {
    /// Active everywhere
    Global,
    /// Active in the sidebar
    Sidebar,
    /// Active in the main panel
    MainPanel,
    /// Active in the register view
    Register,
    /// Active in the budget view
    Budget,
    /// Active in dialogs
    Dialog,
}

/// All keybindings
pub static KEYBINDINGS: &[Keybinding] = &[
    // Global
    Keybinding {
        key: KeyCode::Char('q'),
        modifiers: KeyModifiers::NONE,
        description: "Quit",
        context: KeyContext::Global,
    },
    Keybinding {
        key: KeyCode::Char('?'),
        modifiers: KeyModifiers::NONE,
        description: "Help",
        context: KeyContext::Global,
    },
    Keybinding {
        key: KeyCode::Char(':'),
        modifiers: KeyModifiers::NONE,
        description: "Command palette",
        context: KeyContext::Global,
    },
    Keybinding {
        key: KeyCode::Tab,
        modifiers: KeyModifiers::NONE,
        description: "Switch panel",
        context: KeyContext::Global,
    },
    Keybinding {
        key: KeyCode::Char('h'),
        modifiers: KeyModifiers::NONE,
        description: "Move left/Focus sidebar",
        context: KeyContext::Global,
    },
    Keybinding {
        key: KeyCode::Char('l'),
        modifiers: KeyModifiers::NONE,
        description: "Move right/Focus main",
        context: KeyContext::Global,
    },
    Keybinding {
        key: KeyCode::Char('j'),
        modifiers: KeyModifiers::NONE,
        description: "Move down",
        context: KeyContext::Global,
    },
    Keybinding {
        key: KeyCode::Char('k'),
        modifiers: KeyModifiers::NONE,
        description: "Move up",
        context: KeyContext::Global,
    },
    // Sidebar
    Keybinding {
        key: KeyCode::Char('a'),
        modifiers: KeyModifiers::NONE,
        description: "Add account",
        context: KeyContext::Sidebar,
    },
    Keybinding {
        key: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        description: "Select account",
        context: KeyContext::Sidebar,
    },
    Keybinding {
        key: KeyCode::Char('1'),
        modifiers: KeyModifiers::NONE,
        description: "Accounts view",
        context: KeyContext::Sidebar,
    },
    Keybinding {
        key: KeyCode::Char('2'),
        modifiers: KeyModifiers::NONE,
        description: "Budget view",
        context: KeyContext::Sidebar,
    },
    Keybinding {
        key: KeyCode::Char('3'),
        modifiers: KeyModifiers::NONE,
        description: "Reports view",
        context: KeyContext::Sidebar,
    },
    Keybinding {
        key: KeyCode::Char('A'),
        modifiers: KeyModifiers::SHIFT,
        description: "Toggle archived",
        context: KeyContext::Sidebar,
    },
    // Register
    Keybinding {
        key: KeyCode::Char('a'),
        modifiers: KeyModifiers::NONE,
        description: "Add transaction",
        context: KeyContext::Register,
    },
    Keybinding {
        key: KeyCode::Char('e'),
        modifiers: KeyModifiers::NONE,
        description: "Edit transaction",
        context: KeyContext::Register,
    },
    Keybinding {
        key: KeyCode::Char('c'),
        modifiers: KeyModifiers::NONE,
        description: "Clear transaction",
        context: KeyContext::Register,
    },
    Keybinding {
        key: KeyCode::Char('d'),
        modifiers: KeyModifiers::CONTROL,
        description: "Delete transaction",
        context: KeyContext::Register,
    },
    Keybinding {
        key: KeyCode::Char('v'),
        modifiers: KeyModifiers::NONE,
        description: "Multi-select mode",
        context: KeyContext::Register,
    },
    Keybinding {
        key: KeyCode::Char(' '),
        modifiers: KeyModifiers::NONE,
        description: "Toggle selection",
        context: KeyContext::Register,
    },
    Keybinding {
        key: KeyCode::Char('g'),
        modifiers: KeyModifiers::NONE,
        description: "Go to top",
        context: KeyContext::Register,
    },
    Keybinding {
        key: KeyCode::Char('G'),
        modifiers: KeyModifiers::SHIFT,
        description: "Go to bottom",
        context: KeyContext::Register,
    },
    // Budget
    Keybinding {
        key: KeyCode::Char('['),
        modifiers: KeyModifiers::NONE,
        description: "Previous period",
        context: KeyContext::Budget,
    },
    Keybinding {
        key: KeyCode::Char(']'),
        modifiers: KeyModifiers::NONE,
        description: "Next period",
        context: KeyContext::Budget,
    },
    Keybinding {
        key: KeyCode::Char('m'),
        modifiers: KeyModifiers::NONE,
        description: "Move funds",
        context: KeyContext::Budget,
    },
    Keybinding {
        key: KeyCode::Char('a'),
        modifiers: KeyModifiers::NONE,
        description: "Add category",
        context: KeyContext::Budget,
    },
    Keybinding {
        key: KeyCode::Char('A'),
        modifiers: KeyModifiers::SHIFT,
        description: "Add category group",
        context: KeyContext::Budget,
    },
    // Dialog
    Keybinding {
        key: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        description: "Close dialog",
        context: KeyContext::Dialog,
    },
    Keybinding {
        key: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        description: "Confirm",
        context: KeyContext::Dialog,
    },
];

/// Get keybindings for a specific context
pub fn get_keybindings(context: KeyContext) -> Vec<&'static Keybinding> {
    KEYBINDINGS
        .iter()
        .filter(|kb| kb.context == context || kb.context == KeyContext::Global)
        .collect()
}

/// Format a keybinding for display
pub fn format_keybinding(kb: &Keybinding) -> String {
    let mut parts = Vec::new();

    if kb.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if kb.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if kb.modifiers.contains(KeyModifiers::SHIFT) {
        // Only show Shift for non-character keys
        if !matches!(kb.key, KeyCode::Char(_)) {
            parts.push("Shift");
        }
    }

    let key_str = match kb.key {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        _ => format!("{:?}", kb.key),
    };

    parts.push(&key_str);
    parts.join("+")
}
