//! Budget view
//!
//! Shows budget categories with budgeted, activity, available, and target amounts

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::models::{AccountType, TargetCadence};
use crate::services::{AccountService, BudgetService, CategoryService};
use crate::tui::app::{App, BudgetHeaderDisplay, FocusedPanel};
use crate::tui::layout::BudgetLayout;

/// Render the budget view
pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let layout = BudgetLayout::new(area);

    // Render ATB header
    render_atb_header(frame, app, layout.atb_header);

    // Render category table
    render_category_table(frame, app, layout.categories);
}

/// Render Available to Budget header (or account type balance based on toggle)
fn render_atb_header(frame: &mut Frame, app: &mut App, area: Rect) {
    let budget_service = BudgetService::new(app.storage);

    let (label, amount, color) = match app.budget_header_display {
        BudgetHeaderDisplay::AvailableToBudget => {
            let atb = budget_service
                .get_available_to_budget(&app.current_period)
                .unwrap_or_default();

            let color = if atb.is_negative() {
                Color::Red
            } else if atb.is_zero() {
                Color::Green
            } else {
                Color::Yellow
            };

            let label = if atb.is_negative() {
                "Overspent"
            } else if atb.is_zero() {
                "All money assigned!"
            } else {
                "Available to Budget"
            };

            (label.to_string(), atb, color)
        }
        _ => {
            // Get account type from the display mode
            let account_type = match app.budget_header_display {
                BudgetHeaderDisplay::Checking => AccountType::Checking,
                BudgetHeaderDisplay::Savings => AccountType::Savings,
                BudgetHeaderDisplay::Credit => AccountType::Credit,
                BudgetHeaderDisplay::Cash => AccountType::Cash,
                BudgetHeaderDisplay::Investment => AccountType::Investment,
                BudgetHeaderDisplay::LineOfCredit => AccountType::LineOfCredit,
                BudgetHeaderDisplay::Other => AccountType::Other,
                BudgetHeaderDisplay::AvailableToBudget => unreachable!(),
            };

            let account_service = AccountService::new(app.storage);
            let balance = account_service
                .total_balance_by_type(account_type)
                .unwrap_or_default();
            let count = account_service.count_by_type(account_type).unwrap_or(0);

            // Color: green for positive, red for negative, yellow for zero
            let color = if balance.is_negative() {
                Color::Red
            } else if balance.is_zero() {
                Color::Yellow
            } else {
                Color::Green
            };

            let label = if count == 0 {
                format!("{} (no accounts)", app.budget_header_display.label())
            } else if count == 1 {
                format!("{} (1 account)", app.budget_header_display.label())
            } else {
                format!("{} ({} accounts)", app.budget_header_display.label(), count)
            };

            (label, balance, color)
        }
    };

    // Check for expected income comparison
    let income_indicator = if let Some(overage) = budget_service
        .is_over_expected_income(&app.current_period)
        .ok()
        .flatten()
    {
        // Over budget warning
        Some((format!("  │  Over Income: {} ⚠", overage), Color::Red))
    } else if let Some(remaining) = budget_service
        .get_remaining_to_budget_from_income(&app.current_period)
        .ok()
        .flatten()
    {
        if remaining.is_positive() {
            Some((
                format!("  │  Income Remaining: {} ✓", remaining),
                Color::Green,
            ))
        } else if remaining.is_zero() {
            Some(("  │  Income: Fully Budgeted ✓".to_string(), Color::Green))
        } else {
            None
        }
    } else {
        None
    };

    let block = Block::default()
        .title(format!(" Budget - {} ", app.current_period))
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let mut spans = vec![
        Span::styled("◀ ", Style::default().fg(Color::DarkGray)),
        Span::styled(label, Style::default().fg(Color::White)),
        Span::styled(" ▶  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", amount),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ];

    // Add income indicator if present
    if let Some((income_text, income_color)) = income_indicator {
        spans.push(Span::styled(income_text, Style::default().fg(income_color)));
    }

    spans.extend(vec![
        Span::raw("  │  "),
        Span::styled("[< / >] Toggle  ", Style::default().fg(Color::Yellow)),
        Span::styled("[[ / ]] Period  ", Style::default().fg(Color::Yellow)),
        Span::styled("[m] Move", Style::default().fg(Color::Yellow)),
    ]);

    let line = Line::from(spans);

    let paragraph = Paragraph::new(line).block(block);

    frame.render_widget(paragraph, area);
}

/// Render category budget table
fn render_category_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Main;
    let border_color = if is_focused { Color::Cyan } else { Color::Gray };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let category_service = CategoryService::new(app.storage);
    let budget_service = BudgetService::new(app.storage);

    // Get groups and categories
    let groups = category_service.list_groups().unwrap_or_default();
    let categories = category_service.list_categories().unwrap_or_default();

    // Build rows with group headers
    let mut rows: Vec<Row> = Vec::new();
    let mut row_to_category_index: Vec<Option<usize>> = Vec::new();

    // Track visual index (categories in display order)
    let mut visual_index = 0usize;

    for group in &groups {
        // Group header row
        rows.push(
            Row::new(vec![Cell::from(format!("▼ {}", group.name))])
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .height(1),
        );
        row_to_category_index.push(None);

        // Categories in this group
        let group_categories: Vec<_> = categories
            .iter()
            .filter(|c| c.group_id == group.id)
            .collect();

        for category in group_categories {
            let cat_index = visual_index;
            visual_index += 1;
            let summary = budget_service
                .get_category_summary(category.id, &app.current_period)
                .unwrap_or_else(|_| crate::models::CategoryBudgetSummary::empty(category.id));

            // Get target for this category
            let target = budget_service.get_target(category.id).ok().flatten();

            // Target indicator for category name
            let target_indicator = if target.is_some() { "◉ " } else { "  " };

            // Build target display with progress for ByDate goals
            let target_display = match &target {
                Some(t) => {
                    match &t.cadence {
                        TargetCadence::ByDate { target_date } => {
                            // For ByDate goals, show progress
                            let progress_pct = if t.amount.cents() > 0 {
                                let saved = summary.available.cents().max(0);
                                ((saved as f64 / t.amount.cents() as f64) * 100.0).min(100.0)
                            } else {
                                0.0
                            };
                            format!(
                                "{} by {} ({:.0}%)",
                                t.amount,
                                target_date.format("%b %Y"),
                                progress_pct
                            )
                        }
                        _ => format!("{} {}", t.amount, t.cadence),
                    }
                }
                None => "—".to_string(),
            };

            // Available column styling
            let available_style = if summary.is_overspent() {
                Style::default().fg(Color::Red)
            } else if summary.available.is_zero() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            };

            // Activity styling (negative = spending)
            let activity_style = if summary.activity.is_negative() {
                Style::default().fg(Color::Red)
            } else if summary.activity.is_positive() {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            };

            // Target styling
            let target_style = if target.is_some() {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default().fg(Color::White)
            };

            rows.push(Row::new(vec![
                Cell::from(format!("{}{}", target_indicator, category.name)),
                Cell::from(format!("{}", summary.budgeted)),
                Cell::from(format!("{}", summary.activity)).style(activity_style),
                Cell::from(format!("{}", summary.available)).style(available_style),
                Cell::from(target_display).style(target_style),
            ]));
            row_to_category_index.push(Some(cat_index));
        }
    }

    if rows.is_empty() {
        let text = Paragraph::new("No categories. Run 'envelope category create' to add some.")
            .block(block)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(text, area);
        return;
    }

    // Column widths
    let widths = [
        ratatui::layout::Constraint::Min(20), // Category name (with target indicator)
        ratatui::layout::Constraint::Length(14), // Budgeted
        ratatui::layout::Constraint::Length(14), // Activity
        ratatui::layout::Constraint::Length(14), // Available
        ratatui::layout::Constraint::Length(30), // Target (wider for ByDate progress)
    ];

    // Header row
    let header = Row::new(vec![
        Cell::from("Category").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Budgeted").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Activity").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Available").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Target").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().fg(Color::Yellow))
    .height(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    // Find the row index that corresponds to the selected category index
    let selected_row = row_to_category_index
        .iter()
        .position(|&idx| idx == Some(app.selected_category_index))
        .unwrap_or(0);

    let mut state = TableState::default();
    state.select(Some(selected_row));

    frame.render_stateful_widget(table, area, &mut state);
}
