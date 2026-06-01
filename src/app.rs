use std::collections::HashSet;
use std::path::PathBuf;

use ratatui::widgets::ListState;

use crate::adb::AdbClient;
use crate::file_entry::{Device, FileEntry};

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
            status_message: "adbrowse - press ? for help".into(),
            filter_buffer: String::new(),
        }
    }
}
