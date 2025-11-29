//! Unified budget dialog
//!
//! A tabbed dialog combining:
//! - Period budget editing (set amount for current period)
//! - Target settings (recurring budget goals with cadence)

use chrono::NaiveDate;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::models::{BudgetTarget, CategoryId, Money, TargetCadence};
use crate::services::BudgetService;
use crate::tui::app::App;
use crate::tui::layout::centered_rect_fixed;

/// Which tab is currently active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BudgetTab {
    #[default]
    Period,
    Target,
}

/// Which field is focused in the target tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetField {
    #[default]
    Amount,
    Cadence,
    CustomDays,
    TargetDate,
}

/// Cadence options for budget targets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CadenceOption {
    Weekly,
    #[default]
    Monthly,
    Yearly,
    Custom,
    ByDate,
}

impl CadenceOption {
    pub fn all() -> &'static [Self] {
        &[
            Self::Weekly,
            Self::Monthly,
            Self::Yearly,
            Self::Custom,
            Self::ByDate,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
            Self::Yearly => "Yearly",
            Self::Custom => "Custom (every N days)",
            Self::ByDate => "By Date",
        }
    }
}

/// State for the unified budget dialog
#[derive(Debug, Clone, Default)]
pub struct BudgetDialogState {
    // Common fields
    pub category_id: Option<CategoryId>,
    pub category_name: String,
    pub active_tab: BudgetTab,
    pub error_message: Option<String>,

    // Period tab fields
    pub current_budgeted: Money,
    pub suggested_amount: Option<Money>,
    pub period_amount_input: String,
    pub period_cursor: usize,

    // Target tab fields
    pub has_existing_target: bool,
    pub target_amount_input: String,
    pub target_amount_cursor: usize,
    pub cadence: CadenceOption,
    pub custom_days_input: String,
    pub custom_days_cursor: usize,
    pub target_date_input: String,
    pub target_date_cursor: usize,
    pub target_field: TargetField,
}

impl BudgetDialogState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the dialog for a category
    pub fn init_for_category(
        &mut self,
        category_id: CategoryId,
        category_name: String,
        current_budgeted: Money,
        suggested_amount: Option<Money>,
        existing_target: Option<&BudgetTarget>,
    ) {
        self.category_id = Some(category_id);
        self.category_name = category_name;
        self.active_tab = BudgetTab::Period;
        self.error_message = None;

        // Period tab initialization
        self.current_budgeted = current_budgeted;
        self.suggested_amount = suggested_amount;
        let cents = current_budgeted.cents();
        if cents == 0 {
            self.period_amount_input = String::new();
        } else {
            self.period_amount_input = format!("{:.2}", cents as f64 / 100.0);
        }
        self.period_cursor = self.period_amount_input.len();

        // Target tab initialization
        if let Some(target) = existing_target {
            self.has_existing_target = true;
            let cents = target.amount.cents();
            if cents == 0 {
                self.target_amount_input = String::new();
            } else {
                self.target_amount_input = format!("{:.2}", cents as f64 / 100.0);
            }
            self.target_amount_cursor = self.target_amount_input.len();

            match &target.cadence {
                TargetCadence::Weekly => self.cadence = CadenceOption::Weekly,
                TargetCadence::Monthly => self.cadence = CadenceOption::Monthly,
                TargetCadence::Yearly => self.cadence = CadenceOption::Yearly,
                TargetCadence::Custom { days } => {
                    self.cadence = CadenceOption::Custom;
                    self.custom_days_input = days.to_string();
                    self.custom_days_cursor = self.custom_days_input.len();
                }
                TargetCadence::ByDate { target_date } => {
                    self.cadence = CadenceOption::ByDate;
                    self.target_date_input = target_date.format("%Y-%m-%d").to_string();
                    self.target_date_cursor = self.target_date_input.len();
                }
            }
        } else {
            self.has_existing_target = false;
            self.target_amount_input = String::new();
            self.target_amount_cursor = 0;
            self.cadence = CadenceOption::Monthly;
            self.custom_days_input = "30".to_string();
            self.custom_days_cursor = 2;
            let default_date = chrono::Local::now().date_naive() + chrono::Duration::days(180);
            self.target_date_input = default_date.format("%Y-%m-%d").to_string();
            self.target_date_cursor = self.target_date_input.len();
        }

        self.target_field = TargetField::Amount;
    }

    /// Reset the state
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Switch to the other tab
    pub fn toggle_tab(&mut self) {
        self.active_tab = match self.active_tab {
            BudgetTab::Period => BudgetTab::Target,
            BudgetTab::Target => BudgetTab::Period,
        };
        self.error_message = None;
    }

    /// Fill in the suggested amount (period tab)
    pub fn use_suggested(&mut self) {
        if let Some(suggested) = self.suggested_amount {
            let cents = suggested.cents();
            if cents == 0 {
                self.period_amount_input = String::new();
            } else {
                self.period_amount_input = format!("{:.2}", cents as f64 / 100.0);
            }
            self.period_cursor = self.period_amount_input.len();
            self.error_message = None;
        }
    }

    // Period tab input handling
    pub fn period_insert_char(&mut self, c: char) {
        if c.is_ascii_digit() || c == '.' {
            self.period_amount_input.insert(self.period_cursor, c);
            self.period_cursor += 1;
            self.error_message = None;
        }
    }

    pub fn period_backspace(&mut self) {
        if self.period_cursor > 0 {
            self.period_cursor -= 1;
            self.period_amount_input.remove(self.period_cursor);
            self.error_message = None;
        }
    }

    pub fn period_move_left(&mut self) {
        if self.period_cursor > 0 {
            self.period_cursor -= 1;
        }
    }

    pub fn period_move_right(&mut self) {
        if self.period_cursor < self.period_amount_input.len() {
            self.period_cursor += 1;
        }
    }

    pub fn period_clear(&mut self) {
        self.period_amount_input.clear();
        self.period_cursor = 0;
        self.error_message = None;
    }

    pub fn parse_period_amount(&self) -> Result<Money, String> {
        if self.period_amount_input.trim().is_empty() {
            return Ok(Money::zero());
        }
        Money::parse(&self.period_amount_input).map_err(|_| "Invalid amount format".to_string())
    }

    // Target tab field navigation
    pub fn target_next_field(&mut self) {
        self.target_field = match self.target_field {
            TargetField::Amount => TargetField::Cadence,
            TargetField::Cadence => match self.cadence {
                CadenceOption::Custom => TargetField::CustomDays,
                CadenceOption::ByDate => TargetField::TargetDate,
                _ => TargetField::Amount,
            },
            TargetField::CustomDays => TargetField::Amount,
            TargetField::TargetDate => TargetField::Amount,
        };
    }

    pub fn target_prev_field(&mut self) {
        self.target_field = match self.target_field {
            TargetField::Amount => match self.cadence {
                CadenceOption::Custom => TargetField::CustomDays,
                CadenceOption::ByDate => TargetField::TargetDate,
                _ => TargetField::Cadence,
            },
            TargetField::Cadence => TargetField::Amount,
            TargetField::CustomDays => TargetField::Cadence,
            TargetField::TargetDate => TargetField::Cadence,
        };
    }

    pub fn next_cadence(&mut self) {
        let options = CadenceOption::all();
        let current_idx = options.iter().position(|c| *c == self.cadence).unwrap_or(0);
        let next_idx = (current_idx + 1) % options.len();
        self.cadence = options[next_idx];
    }

    pub fn prev_cadence(&mut self) {
        let options = CadenceOption::all();
        let current_idx = options.iter().position(|c| *c == self.cadence).unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            options.len() - 1
        } else {
            current_idx - 1
        };
        self.cadence = options[prev_idx];
    }

    // Target tab input handling
    pub fn target_insert_char(&mut self, c: char) {
        match self.target_field {
            TargetField::Amount => {
                if c.is_ascii_digit() || c == '.' {
                    self.target_amount_input
                        .insert(self.target_amount_cursor, c);
                    self.target_amount_cursor += 1;
                    self.error_message = None;
                }
            }
            TargetField::CustomDays => {
                if c.is_ascii_digit() {
                    self.custom_days_input.insert(self.custom_days_cursor, c);
                    self.custom_days_cursor += 1;
                    self.error_message = None;
                }
            }
            TargetField::TargetDate => {
                if c.is_ascii_digit() || c == '-' {
                    self.target_date_input.insert(self.target_date_cursor, c);
                    self.target_date_cursor += 1;
                    self.error_message = None;
                }
            }
            TargetField::Cadence => {}
        }
    }

    pub fn target_backspace(&mut self) {
        match self.target_field {
            TargetField::Amount => {
                if self.target_amount_cursor > 0 {
                    self.target_amount_cursor -= 1;
                    self.target_amount_input.remove(self.target_amount_cursor);
                    self.error_message = None;
                }
            }
            TargetField::CustomDays => {
                if self.custom_days_cursor > 0 {
                    self.custom_days_cursor -= 1;
                    self.custom_days_input.remove(self.custom_days_cursor);
                    self.error_message = None;
                }
            }
            TargetField::TargetDate => {
                if self.target_date_cursor > 0 {
                    self.target_date_cursor -= 1;
                    self.target_date_input.remove(self.target_date_cursor);
                    self.error_message = None;
                }
            }
            TargetField::Cadence => {}
        }
    }

    pub fn target_move_left(&mut self) {
        match self.target_field {
            TargetField::Amount => {
                if self.target_amount_cursor > 0 {
                    self.target_amount_cursor -= 1;
                }
            }
            TargetField::CustomDays => {
                if self.custom_days_cursor > 0 {
                    self.custom_days_cursor -= 1;
                }
            }
            TargetField::TargetDate => {
                if self.target_date_cursor > 0 {
                    self.target_date_cursor -= 1;
                }
            }
            TargetField::Cadence => self.prev_cadence(),
        }
    }

    pub fn target_move_right(&mut self) {
        match self.target_field {
            TargetField::Amount => {
                if self.target_amount_cursor < self.target_amount_input.len() {
                    self.target_amount_cursor += 1;
                }
            }
            TargetField::CustomDays => {
                if self.custom_days_cursor < self.custom_days_input.len() {
                    self.custom_days_cursor += 1;
                }
            }
            TargetField::TargetDate => {
                if self.target_date_cursor < self.target_date_input.len() {
                    self.target_date_cursor += 1;
                }
            }
            TargetField::Cadence => self.next_cadence(),
        }
    }

    pub fn target_clear_field(&mut self) {
        match self.target_field {
            TargetField::Amount => {
                self.target_amount_input.clear();
                self.target_amount_cursor = 0;
            }
            TargetField::CustomDays => {
                self.custom_days_input.clear();
                self.custom_days_cursor = 0;
            }
            TargetField::TargetDate => {
                self.target_date_input.clear();
                self.target_date_cursor = 0;
            }
            TargetField::Cadence => {}
        }
        self.error_message = None;
    }

    pub fn parse_target_amount(&self) -> Result<Money, String> {
        if self.target_amount_input.trim().is_empty() {
            return Err("Amount is required".to_string());
        }
        Money::parse(&self.target_amount_input).map_err(|_| "Invalid amount format".to_string())
    }

    pub fn parse_custom_days(&self) -> Result<u32, String> {
        self.custom_days_input
            .parse::<u32>()
            .map_err(|_| "Invalid number of days".to_string())
            .and_then(|d| {
                if d == 0 {
                    Err("Days must be at least 1".to_string())
                } else {
                    Ok(d)
                }
            })
    }

    pub fn parse_target_date(&self) -> Result<NaiveDate, String> {
        NaiveDate::parse_from_str(&self.target_date_input, "%Y-%m-%d")
            .map_err(|_| "Invalid date format (use YYYY-MM-DD)".to_string())
    }

    pub fn build_cadence(&self) -> Result<TargetCadence, String> {
        match self.cadence {
            CadenceOption::Weekly => Ok(TargetCadence::Weekly),
            CadenceOption::Monthly => Ok(TargetCadence::Monthly),
            CadenceOption::Yearly => Ok(TargetCadence::Yearly),
            CadenceOption::Custom => {
                let days = self.parse_custom_days()?;
                Ok(TargetCadence::Custom { days })
            }
            CadenceOption::ByDate => {
                let target_date = self.parse_target_date()?;
                Ok(TargetCadence::ByDate { target_date })
            }
        }
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
    }
}

/// Render the unified budget dialog
pub fn render(frame: &mut Frame, app: &App) {
    let state = &app.budget_dialog_state;

    // Calculate height based on active tab and content
    let height = match state.active_tab {
        BudgetTab::Period => {
            if state.suggested_amount.is_some() {
                13
            } else {
                11
            }
        }
        BudgetTab::Target => match state.cadence {
            CadenceOption::Custom | CadenceOption::ByDate => 15,
            _ => 13,
        },
    };

    let area = centered_rect_fixed(55, height, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" Budget: {} ", state.category_name))
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render tabs and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Length(1), // Separator
            Constraint::Min(0),    // Content
        ])
        .split(inner);

    render_tab_bar(frame, chunks[0], state);

    match state.active_tab {
        BudgetTab::Period => render_period_tab(frame, chunks[2], app),
        BudgetTab::Target => render_target_tab(frame, chunks[2], app),
    }
}

fn render_tab_bar(frame: &mut Frame, area: Rect, state: &BudgetDialogState) {
    let period_style = if state.active_tab == BudgetTab::Period {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        Style::default().fg(Color::White)
    };

    let target_style = if state.active_tab == BudgetTab::Target {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        Style::default().fg(Color::White)
    };

    let target_label = if state.has_existing_target {
        "Target ✓"
    } else {
        "Target"
    };

    let tabs = Line::from(vec![
        Span::raw("  "),
        Span::styled("This Period", period_style),
        Span::raw("    "),
        Span::styled(target_label, target_style),
        Span::raw("          "),
        Span::styled("[Tab]", Style::default().fg(Color::Yellow)),
        Span::styled(" switch", Style::default().fg(Color::White)),
    ]);

    frame.render_widget(Paragraph::new(tabs), area);
}

fn render_period_tab(frame: &mut Frame, area: Rect, app: &App) {
    let state = &app.budget_dialog_state;
    let has_suggested = state.suggested_amount.is_some();

    let constraints = if has_suggested {
        vec![
            Constraint::Length(1), // Period
            Constraint::Length(1), // Current
            Constraint::Length(1), // Suggested
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // New amount label
            Constraint::Length(1), // Amount input
            Constraint::Length(1), // Error
            Constraint::Length(1), // Instructions
            Constraint::Min(0),
        ]
    } else {
        vec![
            Constraint::Length(1), // Period
            Constraint::Length(1), // Current
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // New amount label
            Constraint::Length(1), // Amount input
            Constraint::Length(1), // Error
            Constraint::Length(1), // Instructions
            Constraint::Min(0),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut row = 0;

    // Period
    let period_line = Line::from(vec![
        Span::styled("Period:    ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{}", app.current_period),
            Style::default().fg(Color::White),
        ),
    ]);
    frame.render_widget(Paragraph::new(period_line), chunks[row]);
    row += 1;

    // Current amount
    let current_line = Line::from(vec![
        Span::styled("Current:   ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{}", state.current_budgeted),
            Style::default().fg(Color::White),
        ),
    ]);
    frame.render_widget(Paragraph::new(current_line), chunks[row]);
    row += 1;

    // Suggested amount
    if let Some(suggested) = state.suggested_amount {
        let suggested_line = Line::from(vec![
            Span::styled("Suggested: ", Style::default().fg(Color::Green)),
            Span::styled(
                format!("{}", suggested),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" (from target)", Style::default().fg(Color::DarkGray)),
        ]);
        frame.render_widget(Paragraph::new(suggested_line), chunks[row]);
        row += 1;
    }

    row += 1; // Spacer

    // New amount label
    let label = Line::from(Span::styled(
        "New amount:",
        Style::default().fg(Color::Cyan),
    ));
    frame.render_widget(Paragraph::new(label), chunks[row]);
    row += 1;

    // Amount input with cursor
    let input_line =
        render_input_with_cursor("$", &state.period_amount_input, state.period_cursor, true);
    frame.render_widget(Paragraph::new(input_line), chunks[row]);
    row += 1;

    // Error message
    if let Some(ref error) = state.error_message {
        let error_line = Line::from(Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        ));
        frame.render_widget(Paragraph::new(error_line), chunks[row]);
    }
    row += 1;

    // Instructions
    let mut instructions = vec![
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Save  "),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::raw(" Cancel"),
    ];

    if has_suggested {
        instructions.push(Span::raw("  "));
        instructions.push(Span::styled("[s]", Style::default().fg(Color::Green)));
        instructions.push(Span::raw(" Use Suggested"));
    }

    frame.render_widget(Paragraph::new(Line::from(instructions)), chunks[row]);
}

fn render_target_tab(frame: &mut Frame, area: Rect, app: &App) {
    let state = &app.budget_dialog_state;

    let extra_field = matches!(state.cadence, CadenceOption::Custom | CadenceOption::ByDate);

    let mut constraints = vec![
        Constraint::Length(1), // Amount label+input
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Cadence
    ];

    if extra_field {
        constraints.push(Constraint::Length(1)); // Extra field (days or date)
    }

    constraints.push(Constraint::Length(1)); // Spacer
    constraints.push(Constraint::Length(1)); // Error
    constraints.push(Constraint::Length(1)); // Instructions
    constraints.push(Constraint::Min(0));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut row = 0;

    // Amount field
    render_labeled_input(
        frame,
        chunks[row],
        "Amount",
        "$",
        &state.target_amount_input,
        state.target_amount_cursor,
        state.target_field == TargetField::Amount,
    );
    row += 2; // Skip spacer

    // Cadence selector
    render_selector_field(
        frame,
        chunks[row],
        "Frequency",
        state.cadence.label(),
        state.target_field == TargetField::Cadence,
    );
    row += 1;

    // Extra field for Custom or ByDate
    if extra_field {
        match state.cadence {
            CadenceOption::Custom => {
                render_labeled_input(
                    frame,
                    chunks[row],
                    "Every N days",
                    "",
                    &state.custom_days_input,
                    state.custom_days_cursor,
                    state.target_field == TargetField::CustomDays,
                );
            }
            CadenceOption::ByDate => {
                render_labeled_input(
                    frame,
                    chunks[row],
                    "Target date",
                    "",
                    &state.target_date_input,
                    state.target_date_cursor,
                    state.target_field == TargetField::TargetDate,
                );
            }
            _ => {}
        }
        row += 1;
    }

    row += 1; // Spacer

    // Error message
    if let Some(ref error) = state.error_message {
        let error_line = Line::from(Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        ));
        frame.render_widget(Paragraph::new(error_line), chunks[row]);
    }
    row += 1;

    // Instructions
    let instructions = Line::from(vec![
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Save  "),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::raw(" Cancel  "),
        Span::styled("[Del]", Style::default().fg(Color::Magenta)),
        Span::raw(" Remove  "),
        Span::styled("[j/k]", Style::default().fg(Color::Cyan)),
        Span::raw(" Fields"),
    ]);
    frame.render_widget(Paragraph::new(instructions), chunks[row]);
}

fn render_input_with_cursor(
    prefix: &str,
    value: &str,
    cursor: usize,
    _focused: bool,
) -> Line<'static> {
    let mut spans = vec![];

    if !prefix.is_empty() {
        spans.push(Span::raw(prefix.to_string()));
    }

    let cursor_pos = cursor.min(value.len());
    let (before, after) = value.split_at(cursor_pos);

    spans.push(Span::styled(
        before.to_string(),
        Style::default().fg(Color::White),
    ));

    let cursor_char = after.chars().next().unwrap_or(' ');
    spans.push(Span::styled(
        cursor_char.to_string(),
        Style::default().fg(Color::Black).bg(Color::Cyan),
    ));

    if after.len() > 1 {
        spans.push(Span::styled(
            after[1..].to_string(),
            Style::default().fg(Color::White),
        ));
    }

    Line::from(spans)
}

fn render_labeled_input(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    prefix: &str,
    value: &str,
    cursor: usize,
    focused: bool,
) {
    let label_style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let mut spans = vec![Span::styled(format!("{}: ", label), label_style)];

    if !prefix.is_empty() {
        spans.push(Span::raw(prefix.to_string()));
    }

    if focused {
        let cursor_pos = cursor.min(value.len());
        let (before, after) = value.split_at(cursor_pos);

        spans.push(Span::styled(
            before.to_string(),
            Style::default().fg(Color::White),
        ));

        let cursor_char = after.chars().next().unwrap_or(' ');
        spans.push(Span::styled(
            cursor_char.to_string(),
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ));

        if after.len() > 1 {
            spans.push(Span::styled(
                after[1..].to_string(),
                Style::default().fg(Color::White),
            ));
        }
    } else {
        spans.push(Span::styled(
            value.to_string(),
            Style::default().fg(Color::White),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_selector_field(frame: &mut Frame, area: Rect, label: &str, value: &str, focused: bool) {
    let label_style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let value_style = if focused {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let hint = if focused { " ← h/l →" } else { "" };

    let line = Line::from(vec![
        Span::styled(format!("{}: ", label), label_style),
        Span::styled(format!(" {} ", value), value_style),
        Span::styled(hint.to_string(), Style::default().fg(Color::Yellow)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

/// Handle key events for the budget dialog
pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Esc => {
            app.budget_dialog_state.reset();
            app.close_dialog();
            true
        }

        KeyCode::Tab => {
            app.budget_dialog_state.toggle_tab();
            true
        }

        KeyCode::Enter => {
            match app.budget_dialog_state.active_tab {
                BudgetTab::Period => {
                    if let Err(e) = save_period_budget(app) {
                        app.budget_dialog_state.set_error(e);
                    }
                }
                BudgetTab::Target => {
                    if let Err(e) = save_target(app) {
                        app.budget_dialog_state.set_error(e);
                    }
                }
            }
            true
        }

        KeyCode::Delete => {
            if app.budget_dialog_state.active_tab == BudgetTab::Target {
                if let Err(e) = remove_target(app) {
                    app.budget_dialog_state.set_error(e);
                }
            }
            true
        }

        // Tab-specific handling
        _ => match app.budget_dialog_state.active_tab {
            BudgetTab::Period => handle_period_key(app, key),
            BudgetTab::Target => handle_target_key(app, key),
        },
    }
}

fn handle_period_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::{KeyCode, KeyModifiers};

    match key.code {
        KeyCode::Char('s') => {
            app.budget_dialog_state.use_suggested();
            true
        }

        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.budget_dialog_state.period_clear();
            true
        }

        KeyCode::Char(c) => {
            app.budget_dialog_state.period_insert_char(c);
            true
        }

        KeyCode::Backspace => {
            app.budget_dialog_state.period_backspace();
            true
        }

        KeyCode::Left => {
            app.budget_dialog_state.period_move_left();
            true
        }

        KeyCode::Right => {
            app.budget_dialog_state.period_move_right();
            true
        }

        _ => false,
    }
}

fn handle_target_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::{KeyCode, KeyModifiers};

    match key.code {
        // Field navigation: j/k or up/down arrows
        KeyCode::Down | KeyCode::Char('j') => {
            app.budget_dialog_state.target_next_field();
            true
        }

        KeyCode::Up | KeyCode::Char('k') => {
            app.budget_dialog_state.target_prev_field();
            true
        }

        // Cadence cycling: h/l when on cadence field
        KeyCode::Char('l') if app.budget_dialog_state.target_field == TargetField::Cadence => {
            app.budget_dialog_state.next_cadence();
            true
        }

        KeyCode::Char('h') if app.budget_dialog_state.target_field == TargetField::Cadence => {
            app.budget_dialog_state.prev_cadence();
            true
        }

        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.budget_dialog_state.target_clear_field();
            true
        }

        KeyCode::Char(c) => {
            app.budget_dialog_state.target_insert_char(c);
            true
        }

        KeyCode::Backspace => {
            app.budget_dialog_state.target_backspace();
            true
        }

        KeyCode::Left => {
            app.budget_dialog_state.target_move_left();
            true
        }

        KeyCode::Right => {
            app.budget_dialog_state.target_move_right();
            true
        }

        _ => false,
    }
}

fn save_period_budget(app: &mut App) -> Result<(), String> {
    let state = &app.budget_dialog_state;

    let category_id = state.category_id.ok_or("No category selected")?;
    let amount = state.parse_period_amount()?;

    let budget_service = BudgetService::new(app.storage);
    budget_service
        .assign_to_category(category_id, &app.current_period, amount)
        .map_err(|e| e.to_string())?;

    let cat_name = state.category_name.clone();
    app.budget_dialog_state.reset();
    app.close_dialog();
    app.set_status(format!("Budget for '{}' set to {}", cat_name, amount));

    Ok(())
}

fn save_target(app: &mut App) -> Result<(), String> {
    let state = &app.budget_dialog_state;

    let category_id = state.category_id.ok_or("No category selected")?;
    let amount = state.parse_target_amount()?;
    let cadence = state.build_cadence()?;

    let budget_service = BudgetService::new(app.storage);
    budget_service
        .set_target(category_id, amount, cadence)
        .map_err(|e| e.to_string())?;

    let cat_name = state.category_name.clone();
    app.budget_dialog_state.reset();
    app.close_dialog();
    app.set_status(format!("Budget target set for '{}'", cat_name));

    Ok(())
}

fn remove_target(app: &mut App) -> Result<(), String> {
    let state = &app.budget_dialog_state;

    let category_id = state.category_id.ok_or("No category selected")?;

    let budget_service = BudgetService::new(app.storage);

    if budget_service
        .remove_target(category_id)
        .map_err(|e| e.to_string())?
    {
        let cat_name = state.category_name.clone();
        app.budget_dialog_state.reset();
        app.close_dialog();
        app.set_status(format!("Budget target removed for '{}'", cat_name));
    } else {
        return Err("No target to remove".to_string());
    }

    Ok(())
}
