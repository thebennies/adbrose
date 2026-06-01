use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, InputMode, Modal, Pane};

pub fn handle(app: &mut App, key: KeyEvent) {
    if let Some(modal) = app.modal.clone() {
        handle_modal(app, key, &modal);
        return;
    }

    match app.input_mode {
        InputMode::Normal => handle_normal(app, key),
        InputMode::Filter => handle_filter(app, key),
    }
}

fn handle_modal(app: &mut App, key: KeyEvent, modal: &Modal) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.modal = None;
        }
        _ => match modal {
            Modal::DevicePicker { devices } => {
                handle_device_picker(app, key, devices);
            }
            Modal::ConfirmDelete { .. } => {
                handle_confirm_delete(app, key);
            }
            Modal::Rename { buffer } => {
                let mut buf = buffer.clone();
                handle_text_input(app, key, &mut buf, true);
            }
            Modal::CreateFolder { buffer } => {
                let mut buf = buffer.clone();
                handle_text_input(app, key, &mut buf, false);
            }
            Modal::Error { .. } | Modal::Help | Modal::TransferProgress => {}
        },
    }
}

fn handle_text_input(
    app: &mut App,
    key: KeyEvent,
    buffer: &mut String,
    is_rename: bool,
) {
    match key.code {
        KeyCode::Enter => {
            let text = buffer.clone();
            app.modal = None;
            if text.is_empty() {
                return;
            }
            if is_rename {
                do_rename(app, &text);
            } else {
                do_create_folder(app, &text);
            }
        }
        KeyCode::Backspace => {
            buffer.pop();
            if is_rename {
                app.modal = Some(Modal::Rename { buffer: buffer.clone() });
            } else {
                app.modal = Some(Modal::CreateFolder { buffer: buffer.clone() });
            }
        }
        KeyCode::Char(c) => {
            buffer.push(c);
            if is_rename {
                app.modal = Some(Modal::Rename { buffer: buffer.clone() });
            } else {
                app.modal = Some(Modal::CreateFolder { buffer: buffer.clone() });
            }
        }
        _ => {}
    }
}

fn do_rename(app: &mut App, new_name: &str) {
    let pane = app.active_pane();
    if let Some(entry) = pane.current_entry() {
        let from_path = pane.path.join(&entry.name);
        let to_path = pane.path.join(new_name);
        let result = match app.active_pane {
            Pane::Local => crate::local_fs::rename(&from_path, &to_path),
            Pane::Android => app.adb.rename(
                from_path.to_str().unwrap_or(""),
                to_path.to_str().unwrap_or(""),
            ),
        };
        match result {
            Ok(()) => {
                app.status_message = format!("renamed to {}", new_name);
                let _ = app.refresh_active_pane();
            }
            Err(e) => app.show_error(e.to_string()),
        }
    }
}

fn do_create_folder(app: &mut App, name: &str) {
    let pane = app.active_pane();
    let new_path = pane.path.join(name);
    let result = match app.active_pane {
        Pane::Local => crate::local_fs::mkdir(&new_path),
        Pane::Android => app.adb.mkdir(new_path.to_str().unwrap_or("")),
    };
    match result {
        Ok(()) => {
            app.status_message = format!("created folder {}", name);
            let _ = app.refresh_active_pane();
        }
        Err(e) => app.show_error(e.to_string()),
    }
}

fn handle_device_picker(app: &mut App, key: KeyEvent, devices: &[crate::file_entry::Device]) {
    match key.code {
        KeyCode::Enter => {
            if !devices.is_empty() {
                let serial = devices[0].serial.clone();
                app.adb.serial = Some(serial);
                app.modal = None;
                app.status_message = "device selected".into();
                let _ = app.refresh_active_pane();
            }
        }
        _ => {}
    }
}

fn handle_confirm_delete(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.modal = None;
            execute_delete(app);
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.modal = None;
        }
        _ => {}
    }
}

fn execute_delete(app: &mut App) {
    let selected: Vec<usize> = app.active_pane().selected.iter().cloned().collect();
    let entries_to_delete: Vec<_> = if selected.is_empty() {
        app.active_pane().current_entry().cloned().into_iter().collect()
    } else {
        selected
            .iter()
            .filter_map(|&i| app.active_pane().entries.get(i).cloned())
            .collect()
    };
    let active = app.active_pane;
    let base_path = app.active_pane().path.clone();

    for entry in &entries_to_delete {
        let path = base_path.join(&entry.name);
        let result = match active {
            Pane::Local => crate::local_fs::delete(&path, entry.is_dir()),
            Pane::Android => app.adb.delete(
                path.to_str().unwrap_or(""),
                entry.is_dir(),
            ),
        };
        match result {
            Ok(()) => {
                app.status_message = format!("deleted {}", entry.name);
            }
            Err(e) => {
                app.show_error(format!("failed to delete {}: {}", entry.name, e));
                return;
            }
        }
    }
    let _ = app.refresh_active_pane();
}

fn handle_normal(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Tab => app.switch_pane(),
        KeyCode::Up | KeyCode::Char('k') => app.active_pane_mut().cursor_up(),
        KeyCode::Down | KeyCode::Char('j') => app.active_pane_mut().cursor_down(),
        KeyCode::PageUp => app.active_pane_mut().page_up(),
        KeyCode::PageDown => app.active_pane_mut().page_down(),
        KeyCode::Enter | KeyCode::Char('l') => app.enter_directory(),
        KeyCode::Backspace | KeyCode::Char('h') => app.go_to_parent(),
        KeyCode::Char(' ') => {
            let cursor = app.active_pane().cursor;
            app.active_pane_mut().toggle_select(cursor);
        }
        KeyCode::Char('a') => app.active_pane_mut().select_all(),
        KeyCode::Esc => {
            app.active_pane_mut().clear_selection();
        }
        KeyCode::Char('R') => {
            let _ = app.refresh_active_pane();
        }
        KeyCode::Char('d') => prompt_delete(app),
        KeyCode::Char('r') => prompt_rename(app),
        KeyCode::Char('n') => {
            app.modal = Some(Modal::CreateFolder { buffer: String::new() });
        }
        KeyCode::Char('c') => start_copy(app),
        KeyCode::Char('/') => {
            app.input_mode = InputMode::Filter;
            app.filter_buffer.clear();
        }
        KeyCode::Char('t') => {
            if !app.transfer_queue.jobs.is_empty() {
                app.modal = Some(Modal::TransferProgress);
            }
        }
        KeyCode::Char('x') => {
            if app.transfer_queue.has_active() {
                app.transfer_queue.cancel_active();
                app.status_message = "transfer cancelled".into();
            }
        }
        KeyCode::Char('?') => app.show_help(),
        _ => {}
    }
}

fn prompt_delete(app: &mut App) {
    let pane = app.active_pane();
    let paths: Vec<String> = if pane.selected.is_empty() {
        pane.current_entry().map(|e| e.name.clone()).into_iter().collect()
    } else {
        pane.selected
            .iter()
            .filter_map(|&i| pane.entries.get(i).map(|e| e.name.clone()))
            .collect()
    };
    if paths.is_empty() {
        app.status_message = "nothing selected".into();
        return;
    }
    app.modal = Some(Modal::ConfirmDelete { paths });
}

fn prompt_rename(app: &mut App) {
    let pane = app.active_pane();
    if pane.selected.len() > 1 {
        app.status_message = "select only one item to rename".into();
        return;
    }
    if let Some(entry) = pane.current_entry() {
        app.modal = Some(Modal::Rename {
            buffer: entry.name.clone(),
        });
    }
}

fn start_copy(app: &mut App) {
    let active = app.active_pane;
    let (src_path, src_entries, src_selected, src_cursor) = match active {
        Pane::Local => (
            app.local.path.clone(),
            app.local.entries.clone(),
            app.local.selected.clone(),
            app.local.cursor,
        ),
        Pane::Android => (
            app.android.path.clone(),
            app.android.entries.clone(),
            app.android.selected.clone(),
            app.android.cursor,
        ),
    };
    let dst_path = match active {
        Pane::Local => app.android.path.clone(),
        Pane::Android => app.local.path.clone(),
    };
    let to_android = active == Pane::Local;

    let selected: Vec<usize> = if src_selected.is_empty() {
        vec![src_cursor]
    } else {
        src_selected.iter().cloned().collect()
    };
    let entries: Vec<_> = selected
        .iter()
        .filter_map(|&i| src_entries.get(i).cloned())
        .collect();
    if entries.is_empty() {
        app.status_message = "nothing to copy".into();
        return;
    }

    for entry in &entries {
        let src = src_path.join(&entry.name);
        let dst = dst_path.join(&entry.name);
        app.transfer_queue.add(src, dst, to_android, entry.is_dir());
    }

    app.status_message = format!("queued {} transfer(s)", entries.len());

    process_next_transfer(app);
}

fn process_next_transfer(app: &mut App) {
    if let Some(job) = app.transfer_queue.next_pending() {
        let id = job.id;
        let src = job.source.clone();
        let dst = job.destination.clone();
        let to_android = job.to_android;
        app.transfer_queue.mark_started(id);
        app.status_message = format!(
            "transferring {} -> {}",
            src.display(),
            dst.display()
        );

        let result = if to_android {
            app.adb.push(
                src.to_str().unwrap_or(""),
                dst.to_str().unwrap_or(""),
            )
        } else {
            app.adb.pull(
                src.to_str().unwrap_or(""),
                dst.to_str().unwrap_or(""),
            )
        };

        match result {
            Ok(()) => {
                app.transfer_queue.mark_completed(id);
                app.status_message = format!("transfer complete: {}", src.display());
                let _ = app.refresh_active_pane();
            }
            Err(e) => {
                app.transfer_queue.mark_failed(id, e.to_string());
                app.status_message = format!("transfer failed: {}", e);
            }
        }
    }
}

fn handle_filter(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.active_pane_mut().filter = None;
        }
        KeyCode::Enter => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Backspace => {
            app.filter_buffer.pop();
            let f = if app.filter_buffer.is_empty() {
                None
            } else {
                Some(app.filter_buffer.clone())
            };
            app.active_pane_mut().filter = f;
        }
        KeyCode::Char(c) => {
            app.filter_buffer.push(c);
            app.active_pane_mut().filter = Some(app.filter_buffer.clone());
        }
        _ => {}
    }
}
