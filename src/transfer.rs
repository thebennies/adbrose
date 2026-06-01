use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug)]
#[allow(dead_code)]
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

    pub fn mark_started(&mut self, id: usize) {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.status = TransferStatus::InProgress;
        }
    }

    pub fn mark_completed(&mut self, id: usize) {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.status = TransferStatus::Completed;
        }
    }

    pub fn mark_failed(&mut self, id: usize, error: String) {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.status = TransferStatus::Failed;
            job.error = Some(error);
        }
    }

    pub fn cancel_active(&mut self) -> bool {
        let mut cancelled = false;
        for job in &mut self.jobs {
            if job.status == TransferStatus::InProgress {
                job.status = TransferStatus::Failed;
                job.error = Some("cancelled".into());
                cancelled = true;
            }
        }
        cancelled
    }

    pub fn active_job(&self) -> Option<&TransferJob> {
        self.jobs.iter().find(|j| j.status == TransferStatus::InProgress)
    }

    pub fn recent(&self, count: usize) -> Vec<&TransferJob> {
        self.jobs
            .iter()
            .rev()
            .filter(|j| j.status != TransferStatus::Pending)
            .take(count)
            .collect()
    }

    pub fn next_pending(&self) -> Option<&TransferJob> {
        self.jobs.iter().find(|j| j.status == TransferStatus::Pending)
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum TransferUpdate {
    Started(usize),
    Completed(usize),
    Failed(usize, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_job_gets_id() {
        let mut q = TransferQueue::new();
        let id = q.add(PathBuf::from("/local/file.txt"), PathBuf::from("/sdcard/file.txt"), true, false);
        assert_eq!(id, 0);
        assert_eq!(q.jobs.len(), 1);
        assert_eq!(q.jobs[0].status, TransferStatus::Pending);
    }

    #[test]
    fn multiple_jobs_increasing_ids() {
        let mut q = TransferQueue::new();
        let id0 = q.add(PathBuf::from("/a"), PathBuf::from("/b"), true, false);
        let id1 = q.add(PathBuf::from("/c"), PathBuf::from("/d"), false, true);
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
    }

    #[test]
    fn transition_started() {
        let mut q = TransferQueue::new();
        let id = q.add(PathBuf::from("/a"), PathBuf::from("/b"), true, false);
        q.mark_started(id);
        assert_eq!(q.jobs[0].status, TransferStatus::InProgress);
    }

    #[test]
    fn transition_completed() {
        let mut q = TransferQueue::new();
        let id = q.add(PathBuf::from("/a"), PathBuf::from("/b"), true, false);
        q.mark_started(id);
        q.mark_completed(id);
        assert_eq!(q.jobs[0].status, TransferStatus::Completed);
    }

    #[test]
    fn transition_failed() {
        let mut q = TransferQueue::new();
        let id = q.add(PathBuf::from("/a"), PathBuf::from("/b"), true, false);
        q.mark_started(id);
        q.mark_failed(id, "connection lost".into());
        assert_eq!(q.jobs[0].status, TransferStatus::Failed);
        assert_eq!(q.jobs[0].error.as_deref(), Some("connection lost"));
    }

    #[test]
    fn has_active_checks() {
        let mut q = TransferQueue::new();
        let id = q.add(PathBuf::from("/a"), PathBuf::from("/b"), true, false);
        assert!(!q.has_active());
        q.mark_started(id);
        assert!(q.has_active());
        q.mark_completed(id);
        assert!(!q.has_active());
    }

    #[test]
    fn cancel_active() {
        let mut q = TransferQueue::new();
        let id = q.add(PathBuf::from("/a"), PathBuf::from("/b"), true, false);
        q.mark_started(id);
        let cancelled = q.cancel_active();
        assert!(cancelled);
        assert_eq!(q.jobs[0].status, TransferStatus::Failed);
    }

    #[test]
    fn recent_transfers() {
        let mut q = TransferQueue::new();
        q.add(PathBuf::from("/a"), PathBuf::from("/b"), true, false);
        q.add(PathBuf::from("/c"), PathBuf::from("/d"), true, false);
        q.mark_started(0);
        q.mark_completed(0);
        q.mark_started(1);
        q.mark_failed(1, "err".into());
        let recent = q.recent(5);
        assert_eq!(recent.len(), 2);
    }
}
