//! Budget target dialog

use chrono::NaiveDate;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::models::{CategoryId, Money, TargetCadence};
use crate::services::BudgetService;
use crate::tui::app::App;
use crate::tui::layout::centered_rect_fixed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetField {
    #[default]
    Amount,
    Cadence,
    CustomDays,
    TargetDate,
}

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

#[derive(Debug, Clone)]
pub struct TargetFormState {
    pub category_id: Option<CategoryId>,
    pub category_name: String,
    pub amount_input: String,
    pub amount_cursor: usize,
    pub cadence: CadenceOption,
    pub custom_days_input: String,
    pub custom_days_cursor: usize,
    pub target_date_input: String,
    pub target_date_cursor: usize,
    pub focused_field: TargetField,
    pub error_message: Option<String>,
    pub is_editing: bool,
}

impl Default for TargetFormState {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetFormState {
    pub fn new() -> Self {
        Self {
            category_id: None,
            category_name: String::new(),
            amount_input: String::new(),
            amount_cursor: 0,
            cadence: CadenceOption::Monthly,
            custom_days_input: "30".to_string(),
            custom_days_cursor: 2,
            target_date_input: String::new(),
            target_date_cursor: 0,
            focused_field: TargetField::Amount,
            error_message: None,
            is_editing: false,
        }
    }

    pub fn init_for_category(
        &mut self,
        category_id: CategoryId,
        category_name: String,
        existing_target: Option<&crate::models::BudgetTarget>,
    ) {
        self.category_id = Some(category_id);
        self.category_name = category_name;
        self.focused_field = TargetField::Amount;
        self.error_message = None;

        if let Some(target) = existing_target {
            self.is_editing = true;
            let cents = target.amount.cents();
            if cents == 0 {
                self.amount_input = String::new();
            } else {
                self.amount_input = format!("{:.2}", cents as f64 / 100.0);
            }
            self.amount_cursor = self.amount_input.len();

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
            self.is_editing = false;
            self.amount_input = String::new();
            self.amount_cursor = 0;
            self.cadence = CadenceOption::Monthly;
            self.custom_days_input = "30".to_string();
            self.custom_days_cursor = 2;
            let default_date = chrono::Local::now().date_naive() + chrono::Duration::days(180);
            self.target_date_input = default_date.format("%Y-%m-%d").to_string();
            self.target_date_cursor = self.target_date_input.len();
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn next_field(&mut self) {
        self.focused_field = match self.focused_field {
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

    pub fn prev_field(&mut self) {
        self.focused_field = match self.focused_field {
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

    pub fn insert_char(&mut self, c: char) {
        match self.focused_field {
            TargetField::Amount => {
                if c.is_ascii_digit() || c == '.' {
                    self.amount_input.insert(self.amount_cursor, c);
                    self.amount_cursor += 1;
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

    pub fn backspace(&mut self) {
        match self.focused_field {
            TargetField::Amount => {
                if self.amount_cursor > 0 {
                    self.amount_cursor -= 1;
                    self.amount_input.remove(self.amount_cursor);
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

    pub fn move_left(&mut self) {
        match self.focused_field {
            TargetField::Amount => {
                if self.amount_cursor > 0 {
                    self.amount_cursor -= 1;
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

    pub fn move_right(&mut self) {
        match self.focused_field {
            TargetField::Amount => {
                if self.amount_cursor < self.amount_input.len() {
                    self.amount_cursor += 1;
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

    pub fn parse_amount(&self) -> Result<Money, String> {
        if self.amount_input.trim().is_empty() {
            return Err("Amount is required".to_string());
        }
        Money::parse(&self.amount_input).map_err(|_| "Invalid amount format".to_string())
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

pub fn render(frame: &mut Frame, app: &App) {
    let height = match app.target_form.cadence {
        CadenceOption::Custom | CadenceOption::ByDate => 14,
        _ => 12,
    };
    let area = centered_rect_fixed(55, height, frame.area());

    frame.render_widget(Clear, area);

    let state = &app.target_form;

    let title = if state.is_editing {
        " Edit Budget Target "
    } else {
        " Set Budget Target "
    };

    let block = Block::default()
        .title(title)
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut constraints = vec![
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ];

    match state.cadence {
        CadenceOption::Custom | CadenceOption::ByDate => {
            constraints.push(Constraint::Length(1));
            constraints.push(Constraint::Length(1));
        }
        _ => {}
    }

    constraints.push(Constraint::Length(1));
    constraints.push(Constraint::Length(1));
    constraints.push(Constraint::Min(0));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let mut row = 0;

    let category_line = Line::from(vec![
        Span::styled("Category: ", Style::default().fg(Color::Yellow)),
        Span::styled(&state.category_name, Style::default().fg(Color::White)),
    ]);
    frame.render_widget(Paragraph::new(category_line), chunks[row]);
    row += 2;

    render_input_field(
        frame,
        chunks[row],
        "Amount",
        "$",
        &state.amount_input,
        state.amount_cursor,
        state.focused_field == TargetField::Amount,
    );
    row += 2;

    render_selector_field(
        frame,
        chunks[row],
        "Frequency",
        state.cadence.label(),
        state.focused_field == TargetField::Cadence,
    );
    row += 1;

    match state.cadence {
        CadenceOption::Custom => {
            row += 1;
            render_input_field(
                frame,
                chunks[row],
                "Every N days",
                "",
                &state.custom_days_input,
                state.custom_days_cursor,
                state.focused_field == TargetField::CustomDays,
            );
            row += 1;
        }
        CadenceOption::ByDate => {
            row += 1;
            render_input_field(
                frame,
                chunks[row],
                "Target date",
                "",
                &state.target_date_input,
                state.target_date_cursor,
                state.focused_field == TargetField::TargetDate,
            );
            row += 1;
        }
        _ => {}
    }

    if let Some(ref error) = state.error_message {
        let error_line = Line::from(Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        ));
        frame.render_widget(Paragraph::new(error_line), chunks[row]);
    }
    row += 1;

    let instructions = Line::from(vec![
        Span::styled("[Tab]", Style::default().fg(Color::Yellow)),
        Span::raw(" Next  "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Save  "),
        Span::styled("[Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Cancel  "),
        Span::styled("[Del]", Style::default().fg(Color::Magenta)),
        Span::raw(" Remove"),
    ]);
    frame.render_widget(Paragraph::new(instructions), chunks[row]);
}

fn render_input_field(
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
        spans.push(Span::raw(prefix));
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

    let hint = if focused { " <- j/k ->" } else { "" };

    let line = Line::from(vec![
        Span::styled(format!("{}: ", label), label_style),
        Span::styled(format!(" {} ", value), value_style),
        Span::styled(hint, Style::default().fg(Color::Yellow)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    use crossterm::event::{KeyCode, KeyModifiers};

    match key.code {
        KeyCode::Esc => {
            app.target_form.reset();
            app.close_dialog();
            true
        }

        KeyCode::Tab | KeyCode::Down => {
            app.target_form.next_field();
            true
        }

        KeyCode::BackTab | KeyCode::Up => {
            app.target_form.prev_field();
            true
        }

        KeyCode::Enter => {
            if let Err(e) = save_target(app) {
                app.target_form.set_error(e);
            }
            true
        }

        KeyCode::Delete => {
            if let Err(e) = remove_target(app) {
                app.target_form.set_error(e);
            }
            true
        }

        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            match app.target_form.focused_field {
                TargetField::Amount => {
                    app.target_form.amount_input.clear();
                    app.target_form.amount_cursor = 0;
                }
                TargetField::CustomDays => {
                    app.target_form.custom_days_input.clear();
                    app.target_form.custom_days_cursor = 0;
                }
                TargetField::TargetDate => {
                    app.target_form.target_date_input.clear();
                    app.target_form.target_date_cursor = 0;
                }
                TargetField::Cadence => {}
            }
            true
        }

        KeyCode::Char('j') if app.target_form.focused_field == TargetField::Cadence => {
            app.target_form.next_cadence();
            true
        }

        KeyCode::Char('k') if app.target_form.focused_field == TargetField::Cadence => {
            app.target_form.prev_cadence();
            true
        }

        KeyCode::Char(c) => {
            app.target_form.insert_char(c);
            true
        }

        KeyCode::Backspace => {
            app.target_form.backspace();
            true
        }

        KeyCode::Left => {
            app.target_form.move_left();
            true
        }

        KeyCode::Right => {
            app.target_form.move_right();
            true
        }

        _ => false,
    }
}

fn save_target(app: &mut App) -> Result<(), String> {
    let state = &app.target_form;

    let category_id = state.category_id.ok_or("No category selected")?;
    let amount = state.parse_amount()?;
    let cadence = state.build_cadence()?;

    let budget_service = BudgetService::new(app.storage);

    budget_service
        .set_target(category_id, amount, cadence)
        .map_err(|e| e.to_string())?;

    let cat_name = state.category_name.clone();
    app.target_form.reset();
    app.close_dialog();
    app.set_status(format!("Budget target set for '{}'", cat_name));

    Ok(())
}

fn remove_target(app: &mut App) -> Result<(), String> {
    let state = &app.target_form;

    let category_id = state.category_id.ok_or("No category selected")?;

    let budget_service = BudgetService::new(app.storage);

    if budget_service
        .remove_target(category_id)
        .map_err(|e| e.to_string())?
    {
        let cat_name = state.category_name.clone();
        app.target_form.reset();
        app.close_dialog();
        app.set_status(format!("Budget target removed for '{}'", cat_name));
    } else {
        return Err("No target to remove".to_string());
    }

    Ok(())
}
