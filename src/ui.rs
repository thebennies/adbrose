use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, List, ListItem, Paragraph,
};

use crate::app::{App, InputMode, Modal, Pane};
use crate::file_entry::{format_size, FileKind};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    draw_panes(frame, app, chunks[0]);
    draw_transfer_bar(frame, app, chunks[1]);
    draw_status_bar(frame, app, chunks[2]);

    if let Some(ref modal) = app.modal {
        draw_modal(frame, app, modal);
    }
}

fn draw_panes(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_pane(frame, app, Pane::Local, &app.local, chunks[0]);
    draw_pane(frame, app, Pane::Android, &app.android, chunks[1]);
}

fn draw_pane(
    frame: &mut Frame,
    app: &App,
    pane: Pane,
    state: &crate::app::PaneState,
    area: Rect,
) {
    let is_active = app.active_pane == pane;
    let border_color = if is_active { Color::Cyan } else { Color::DarkGray };
    let label = match pane {
        Pane::Local => "Local",
        Pane::Android => "Android",
    };
    let title = format!(" {} {} ", label, state.path.display());

    let items: Vec<ListItem> = if state.loading {
        vec![ListItem::new(Line::from("  Loading..."))]
    } else if state.entries.is_empty() {
        vec![ListItem::new(Line::from("  (empty)"))]
    } else {
        let filtered = state.filtered_entries();
        if filtered.is_empty() {
            vec![ListItem::new(Line::from("  (no matches)"))]
        } else {
            filtered
                .iter()
                .map(|(idx, entry)| {
                    let is_selected = state.selected.contains(idx);
                    let marker = if entry.kind == FileKind::Directory { "/" } else { "" };
                    let size_str = if entry.is_dir() { String::new() } else { format_size(entry.size) };

                    let name_style = if is_selected {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else if entry.is_dir() {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let width = area.width.saturating_sub(4) as usize;
                    let name_width = width.saturating_sub(10);
                    let display_name = if entry.name.len() > name_width {
                        let end = name_width.saturating_sub(1);
                        format!("{}...", &entry.name[..end])
                    } else {
                        format!("{:<width$}", entry.name, width = name_width)
                    };

                    let line = Line::from(vec![
                        Span::styled(format!(" {}{}", display_name, marker), name_style),
                        Span::styled(format!("{:>10}", size_str), Style::default().fg(Color::DarkGray)),
                    ]);
                    ListItem::new(line)
                })
                .collect()
        }
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title)
                .title_style(
                    Style::default()
                        .fg(if is_active { Color::Cyan } else { Color::DarkGray })
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut list_state = state.list_state.clone();
    list_state.select(Some(state.cursor));
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_transfer_bar(frame: &mut Frame, app: &App, area: Rect) {
    let text = if let Some(job) = app.transfer_queue.active_job() {
        format!(" Transferring: {} -> {}", job.source.display(), job.destination.display())
    } else {
        let completed = app.transfer_queue.jobs.iter().filter(|j| j.status == crate::transfer::TransferStatus::Completed).count();
        let failed = app.transfer_queue.jobs.iter().filter(|j| j.status == crate::transfer::TransferStatus::Failed).count();
        if completed + failed > 0 {
            format!(" Transfers: {} completed, {} failed", completed, failed)
        } else {
            String::new()
        }
    };

    let bar = Paragraph::new(Line::from(text)).style(
        Style::default().bg(Color::DarkGray).fg(Color::Yellow),
    );
    frame.render_widget(bar, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (left, right) = match app.input_mode {
        InputMode::Normal => (
            format!(" {}", app.status_message),
            " q:quit  Tab:switch  c:copy  d:delete  ?:help ".to_string(),
        ),
        InputMode::Filter => (
            format!(" Filter: {}", app.filter_buffer),
            " Enter/Esc:done ".to_string(),
        ),
    };

    let line = Line::from(vec![
        Span::styled(left, Style::default().bg(Color::Blue).fg(Color::White)),
        Span::styled(right, Style::default().bg(Color::DarkGray).fg(Color::White)),
    ]);
    let bar = Paragraph::new(line);
    frame.render_widget(bar, area);
}

fn draw_modal(frame: &mut Frame, app: &App, modal: &Modal) {
    let area = centered_rect(60, 50, frame.area());
    frame.render_widget(Clear, area);

    match modal {
        Modal::Help => draw_help(frame, area),
        Modal::Error { message } => draw_error(frame, area, message),
        Modal::DevicePicker { devices } => draw_device_picker(frame, area, devices),
        Modal::ConfirmDelete { paths } => draw_confirm_delete(frame, area, paths),
        Modal::Rename { buffer } => draw_rename(frame, area, buffer),
        Modal::CreateFolder { buffer } => draw_create_folder(frame, area, buffer),
        Modal::TransferProgress => draw_transfer_progress(frame, area, app),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(Span::styled(
            " adbrowse - Help",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(" Tab          Switch pane"),
        Line::from(" Up/k  Down/j  Move selection"),
        Line::from(" PgUp/PgDn    Scroll faster"),
        Line::from(" Enter/l      Enter directory"),
        Line::from(" Backspace/h  Parent directory"),
        Line::from(" Space        Toggle selection"),
        Line::from(" a            Select all"),
        Line::from(" Esc          Clear selection"),
        Line::from(" c            Copy to opposite pane"),
        Line::from(" d            Delete (with confirm)"),
        Line::from(" r            Rename selected"),
        Line::from(" n            Create new folder"),
        Line::from(" /            Filter/search"),
        Line::from(" R            Refresh pane"),
        Line::from(" t            Show transfer queue"),
        Line::from(" x            Cancel active transfer"),
        Line::from(" ?            This help"),
        Line::from(" q            Quit"),
        Line::from(""),
        Line::from(Span::styled(
            " Press Esc or q to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

fn draw_error(frame: &mut Frame, area: Rect, message: &str) {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!(" Error: {}", message),
            Style::default().fg(Color::Red),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " Press Esc to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Red)))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

fn draw_device_picker(frame: &mut Frame, area: Rect, devices: &[crate::file_entry::Device]) {
    let mut text = vec![
        Line::from(Span::styled(
            " Select Device",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    for d in devices {
        let state_icon = if d.state == "device" { "[ok]" } else { "[?]" };
        text.push(Line::from(format!(" {} {} ({})", state_icon, d.serial, d.state)));
    }
    text.push(Line::from(""));
    text.push(Line::from(Span::styled(
        " Enter: select  Esc: quit",
        Style::default().fg(Color::DarkGray),
    )));
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Devices "))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

fn draw_confirm_delete(frame: &mut Frame, area: Rect, paths: &[String]) {
    let mut text = vec![
        Line::from(Span::styled(
            " Confirm Delete",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
        )),
        Line::from(""),
    ];
    for p in paths.iter().take(10) {
        text.push(Line::from(format!("  {}", p)));
    }
    if paths.len() > 10 {
        text.push(Line::from(format!("  ... and {} more", paths.len() - 10)));
    }
    text.push(Line::from(""));
    text.push(Line::from(" y: confirm  n/Esc: cancel"));
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Red)))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

fn draw_rename(frame: &mut Frame, area: Rect, buffer: &str) {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(" Rename to:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(format!(" {}", buffer)),
        Line::from("|"),
        Line::from(""),
        Line::from(Span::styled(
            " Enter: confirm  Esc: cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Rename "))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

fn draw_create_folder(frame: &mut Frame, area: Rect, buffer: &str) {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(" New folder name:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(format!(" {}", buffer)),
        Line::from("|"),
        Line::from(""),
        Line::from(Span::styled(
            " Enter: confirm  Esc: cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" New Folder "))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

fn draw_transfer_progress(frame: &mut Frame, area: Rect, app: &App) {
    let mut text = vec![
        Line::from(Span::styled(
            " Transfer Queue",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    for job in app.transfer_queue.recent(10) {
        let icon = match job.status {
            crate::transfer::TransferStatus::Pending => "...",
            crate::transfer::TransferStatus::InProgress => ">>",
            crate::transfer::TransferStatus::Completed => "[ok]",
            crate::transfer::TransferStatus::Failed => "[!!]",
        };
        text.push(Line::from(format!(
            " {} {} -> {}",
            icon,
            job.source.display(),
            job.destination.display()
        )));
    }
    text.push(Line::from(""));
    text.push(Line::from(Span::styled(
        " Press Esc to close",
        Style::default().fg(Color::DarkGray),
    )));
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Transfers "))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}
