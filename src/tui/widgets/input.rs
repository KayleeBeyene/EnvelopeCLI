//! Text input widget
//!
//! A text input field with cursor support

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};

/// A simple text input widget
#[derive(Debug, Clone)]
pub struct TextInput {
    /// Current text content
    pub content: String,
    /// Cursor position
    pub cursor: usize,
    /// Whether the input is focused
    pub focused: bool,
    /// Placeholder text
    pub placeholder: String,
    /// Label
    pub label: String,
}

impl TextInput {
    /// Create a new text input
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
            focused: false,
            placeholder: String::new(),
            label: String::new(),
        }
    }

    /// Set the label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Set the placeholder
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set focused state
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Set content
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self.cursor = self.content.len();
        self
    }

    /// Insert a character at the cursor
    pub fn insert(&mut self, c: char) {
        self.content.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.content.remove(self.cursor);
        }
    }

    /// Delete character at cursor
    pub fn delete(&mut self) {
        if self.cursor < self.content.len() {
            self.content.remove(self.cursor);
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        if self.cursor < self.content.len() {
            self.cursor += 1;
        }
    }

    /// Move cursor to start
    pub fn move_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end
    pub fn move_end(&mut self) {
        self.cursor = self.content.len();
    }

    /// Clear the content
    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor = 0;
    }

    /// Get the current content
    pub fn value(&self) -> &str {
        &self.content
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate areas
        let label_width = if self.label.is_empty() {
            0
        } else {
            self.label.len() + 2
        };

        let input_start = area.x + label_width as u16;
        let _input_width = area.width.saturating_sub(label_width as u16);

        // Render label if present
        if !self.label.is_empty() {
            let label_line = Line::from(vec![
                Span::styled(&self.label, Style::default().fg(Color::Cyan)),
                Span::raw(": "),
            ]);
            buf.set_line(area.x, area.y, &label_line, label_width as u16);
        }

        // Determine display text
        let display_text = if self.content.is_empty() && !self.focused {
            self.placeholder.clone()
        } else {
            self.content.clone()
        };

        let text_style = if self.content.is_empty() && !self.focused {
            Style::default().fg(Color::Yellow)
        } else if self.focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Yellow)
        };

        // Render text
        buf.set_string(input_start, area.y, &display_text, text_style);

        // Render cursor if focused
        if self.focused {
            let cursor_x = input_start + self.cursor as u16;
            if cursor_x < area.x + area.width {
                let cursor_char = if self.cursor < self.content.len() {
                    self.content.chars().nth(self.cursor).unwrap_or('_')
                } else {
                    '_'
                };
                buf.set_string(
                    cursor_x,
                    area.y,
                    cursor_char.to_string(),
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                );
            }
        }
    }
}
