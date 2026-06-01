use std::collections::HashSet;
use std::path::PathBuf;

#[allow(unused_imports)]
use ratatui::widgets::ListState;

use crate::adb::AdbClient;
use crate::file_entry::{Device, FileEntry};
use crate::transfer::TransferQueue;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pane {
    Local,
    Android,
}

#[derive(Debug, Clone)]
pub enum Modal {
    Help,
    Error { message: String },
    DevicePicker { devices: Vec<Device> },
    ConfirmDelete { paths: Vec<String> },
    Rename { buffer: String },
    CreateFolder { buffer: String },
    TransferProgress,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Filter,
}

#[derive(Debug)]
pub struct PaneState {
    pub path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: HashSet<usize>,
    pub cursor: usize,
    pub loading: bool,
    pub filter: Option<String>,
    pub list_state: ListState,
}

impl PaneState {
    pub fn new(path: PathBuf) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            path,
            entries: Vec::new(),
            selected: HashSet::new(),
            cursor: 0,
            loading: false,
            filter: None,
            list_state,
        }
    }

    pub fn filtered_entries(&self) -> Vec<(usize, &FileEntry)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                self.filter
                    .as_ref()
                    .map_or(true, |f| e.name.to_lowercase().contains(&f.to_lowercase()))
            })
            .collect()
    }

    pub fn current_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.cursor)
    }

    pub fn toggle_select(&mut self, index: usize) {
        if self.selected.contains(&index) {
            self.selected.remove(&index);
        } else {
            self.selected.insert(index);
        }
    }

    pub fn select_all(&mut self) {
        self.selected = (0..self.entries.len()).collect();
    }

    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    pub fn cursor_down(&mut self) {
        if !self.entries.is_empty() {
            self.cursor = (self.cursor + 1).min(self.entries.len() - 1);
        }
    }

    pub fn cursor_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn page_down(&mut self) {
        let step = 10;
        self.cursor = (self.cursor + step).min(self.entries.len().saturating_sub(1));
    }

    pub fn page_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(10);
    }

    pub fn set_entries(&mut self, entries: Vec<FileEntry>) {
        self.entries = entries;
        self.cursor = self.cursor.min(self.entries.len().saturating_sub(1));
        self.loading = false;
    }

    pub fn navigate_to(&mut self, path: PathBuf) {
        self.path = path;
        self.cursor = 0;
        self.selected.clear();
        self.filter = None;
        self.loading = true;
    }
}

pub struct App {
    pub active_pane: Pane,
    pub local: PaneState,
    pub android: PaneState,
    pub adb: AdbClient,
    pub modal: Option<Modal>,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub status_message: String,
    pub filter_buffer: String,
    pub transfer_queue: TransferQueue,
}

impl App {
    pub fn new(adb: AdbClient, local_path: PathBuf) -> Self {
        Self {
            active_pane: Pane::Local,
            local: PaneState::new(local_path),
            android: PaneState::new(PathBuf::from("/sdcard")),
            adb,
            modal: None,
            input_mode: InputMode::Normal,
            should_quit: false,
            status_message: "abdrose - press ? for help".into(),
            filter_buffer: String::new(),
            transfer_queue: TransferQueue::new(),
        }
    }

    pub fn active_pane(&self) -> &PaneState {
        match self.active_pane {
            Pane::Local => &self.local,
            Pane::Android => &self.android,
        }
    }

    pub fn active_pane_mut(&mut self) -> &mut PaneState {
        match self.active_pane {
            Pane::Local => &mut self.local,
            Pane::Android => &mut self.android,
        }
    }

    #[allow(dead_code)]
    pub fn inactive_pane(&self) -> &PaneState {
        match self.active_pane {
            Pane::Local => &self.android,
            Pane::Android => &self.local,
        }
    }

    pub fn switch_pane(&mut self) {
        self.active_pane = match self.active_pane {
            Pane::Local => Pane::Android,
            Pane::Android => Pane::Local,
        };
        self.status_message = format!("switched to {} pane", match self.active_pane {
            Pane::Local => "local",
            Pane::Android => "android",
        });
    }

    pub fn refresh_active_pane(&mut self) -> Result<(), String> {
        let entries = match self.active_pane {
            Pane::Local => crate::local_fs::list_dir(&self.local.path),
            Pane::Android => self.adb.list_dir(
                self.android.path.to_str().unwrap_or("/sdcard"),
            ),
        };
        match entries {
            Ok(e) => {
                self.active_pane_mut().set_entries(e);
                self.status_message = "refreshed".into();
                Ok(())
            }
            Err(e) => {
                self.status_message = format!("error: {}", e);
                Err(e.to_string())
            }
        }
    }

    pub fn enter_directory(&mut self) {
        let pane = self.active_pane();
        if let Some(entry) = pane.current_entry() {
            if entry.is_dir() {
                let new_path = pane.path.join(&entry.name);
                self.active_pane_mut().navigate_to(new_path);
                let _ = self.refresh_active_pane();
            }
        }
    }

    pub fn go_to_parent(&mut self) {
        let pane = self.active_pane();
        if let Some(parent) = pane.path.parent() {
            let parent = parent.to_path_buf();
            self.active_pane_mut().navigate_to(parent);
            let _ = self.refresh_active_pane();
        }
    }

    pub fn show_error(&mut self, msg: impl Into<String>) {
        self.modal = Some(Modal::Error { message: msg.into() });
    }

    pub fn show_help(&mut self) {
        self.modal = Some(Modal::Help);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_entry::{FileEntry, FileKind};

    fn make_entries() -> Vec<FileEntry> {
        vec![
            FileEntry { name: "dir1".into(), kind: FileKind::Directory, size: 0, modified: None },
            FileEntry { name: "file1.txt".into(), kind: FileKind::File, size: 100, modified: None },
            FileEntry { name: "file2.txt".into(), kind: FileKind::File, size: 200, modified: None },
        ]
    }

    #[test]
    fn toggle_selection() {
        let mut pane = PaneState::new(PathBuf::from("/tmp"));
        pane.entries = make_entries();
        pane.toggle_select(1);
        assert!(pane.selected.contains(&1));
        pane.toggle_select(1);
        assert!(!pane.selected.contains(&1));
    }

    #[test]
    fn select_all() {
        let mut pane = PaneState::new(PathBuf::from("/tmp"));
        pane.entries = make_entries();
        pane.select_all();
        assert_eq!(pane.selected.len(), 3);
    }

    #[test]
    fn clear_selection() {
        let mut pane = PaneState::new(PathBuf::from("/tmp"));
        pane.entries = make_entries();
        pane.select_all();
        pane.clear_selection();
        assert!(pane.selected.is_empty());
    }

    #[test]
    fn move_cursor_bounds() {
        let mut pane = PaneState::new(PathBuf::from("/tmp"));
        pane.entries = make_entries();
        pane.cursor_down();
        assert_eq!(pane.cursor, 1);
        pane.cursor_down();
        pane.cursor_down();
        assert_eq!(pane.cursor, 2);
        pane.cursor_up();
        pane.cursor_up();
        pane.cursor_up();
        assert_eq!(pane.cursor, 0);
    }

    #[test]
    fn filtered_entries_match() {
        let mut pane = PaneState::new(PathBuf::from("/tmp"));
        pane.entries = make_entries();
        pane.filter = Some("file".into());
        let filtered = pane.filtered_entries();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn active_pane_switches() {
        let adb = crate::adb::AdbClient { serial: None };
        let mut app = App::new(adb, PathBuf::from("/tmp"));
        assert_eq!(app.active_pane, Pane::Local);
        app.switch_pane();
        assert_eq!(app.active_pane, Pane::Android);
        app.switch_pane();
        assert_eq!(app.active_pane, Pane::Local);
    }
}
