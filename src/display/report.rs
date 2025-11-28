//! Report formatting utilities for terminal output
//!
//! Provides formatting helpers for various report types.

use crate::models::Money;

/// Format a money amount with color hints for terminal display
pub fn format_money_colored(amount: Money) -> String {
    if amount.is_negative() {
        format!("\x1b[31m{}\x1b[0m", amount) // Red for negative
    } else if amount.is_positive() {
        format!("\x1b[32m{}\x1b[0m", amount) // Green for positive
    } else {
        amount.to_string()
    }
}

/// Format a percentage with appropriate precision
pub fn format_percentage(pct: f64) -> String {
    if pct < 0.1 && pct > 0.0 {
        format!("{:.2}%", pct)
    } else if pct < 10.0 {
        format!("{:.1}%", pct)
    } else {
        format!("{:.0}%", pct)
    }
}

/// Create a simple bar chart representation
pub fn format_bar(value: f64, max_value: f64, width: usize) -> String {
    if max_value <= 0.0 || value <= 0.0 {
        return " ".repeat(width);
    }

    let filled = ((value / max_value) * width as f64).round() as usize;
    let filled = filled.min(width);

    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}

/// Format a header line with padding
pub fn format_header(title: &str, width: usize) -> String {
    let padding = if title.len() >= width {
        0
    } else {
        (width - title.len()) / 2
    };
    format!("{}{}", " ".repeat(padding), title)
}

/// Format a separator line
pub fn separator(width: usize) -> String {
    "─".repeat(width)
}

/// Format a double separator line
pub fn double_separator(width: usize) -> String {
    "═".repeat(width)
}

/// Truncate a string to a maximum length with ellipsis
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        "...".chars().take(max_len).collect()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Right-align text in a field of given width
pub fn right_align(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{:>width$}", s, width = width)
    }
}

/// Left-align text in a field of given width
pub fn left_align(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{:<width$}", s, width = width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_percentage() {
        assert_eq!(format_percentage(0.05), "0.05%");
        assert_eq!(format_percentage(5.5), "5.5%");
        assert_eq!(format_percentage(50.0), "50%");
    }

    #[test]
    fn test_format_bar() {
        let bar = format_bar(50.0, 100.0, 10);
        assert_eq!(bar.chars().filter(|c| *c == '█').count(), 5);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Hello World", 5), "He...");
        assert_eq!(truncate("Hi", 5), "Hi");
        assert_eq!(truncate("Test", 4), "Test");
    }

    #[test]
    fn test_alignment() {
        assert_eq!(right_align("abc", 5), "  abc");
        assert_eq!(left_align("abc", 5), "abc  ");
    }
}
