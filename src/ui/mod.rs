use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::App;

/// Top-level draw: splits the terminal into a main area + status bar.
pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    draw_main(frame, app, chunks[0]);
    draw_status_bar(frame, app, chunks[1]);
}

fn draw_main(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    draw_sidebar(frame, app, chunks[0]);
    draw_content(frame, app, chunks[1]);
}

fn draw_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|i| ListItem::new(Line::from(i.as_str())))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Items ")
                .title_style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(app.cursor_index));

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_content(frame: &mut Frame, app: &App, area: Rect) {
    let selected = app
        .items
        .get(app.cursor_index)
        .map(|s| s.as_str())
        .unwrap_or("(none)");

    let paragraph = Paragraph::new(format!("Selected: {selected}\n\nUse ↑/k and ↓/j to navigate.\nPress q or Esc to quit."))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Content ")
                .title_style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(Line::from(format!(
        " {} | ↑/k ↓/j navigate | q quit | {} ",
        app.title,
        app.items.get(app.cursor_index).unwrap_or(&"?".into()),
    )))
    .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(status, area);
}
