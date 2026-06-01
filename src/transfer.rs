use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug)]
pub struct TransferJob {
    pub id: usize,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub to_android: bool,
    pub is_dir: bool,
    pub status: TransferStatus,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct TransferQueue {
    pub jobs: Vec<TransferJob>,
    pub next_id: usize,
}

impl TransferQueue {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            next_id: 0,
        }
    }

    pub fn add(&mut self, source: PathBuf, destination: PathBuf, to_android: bool, is_dir: bool) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.jobs.push(TransferJob {
            id,
            source,
            destination,
            to_android,
            is_dir,
            status: TransferStatus::Pending,
            error: None,
        });
        id
    }

    pub fn has_active(&self) -> bool {
        self.jobs.iter().any(|j| j.status == TransferStatus::InProgress)
    }
}

#[derive(Debug)]
pub enum TransferUpdate {
    Started(usize),
    Completed(usize),
    Failed(usize, String),
}
