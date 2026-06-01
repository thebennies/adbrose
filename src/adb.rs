use crate::error::Result;
use crate::file_entry::{Device, FileEntry};

pub struct AdbClient {
    pub serial: Option<String>,
}

impl AdbClient {
    pub fn new(serial: Option<String>) -> Result<Self> {
        Ok(Self { serial })
    }

    pub fn device_list(&self) -> Result<Vec<Device>> {
        Ok(vec![])
    }

    pub fn list_dir(&self, path: &str) -> Result<Vec<FileEntry>> {
        let _ = path;
        Ok(vec![])
    }

    pub fn push(&self, local: &str, remote: &str) -> Result<()> {
        let _ = (local, remote);
        Ok(())
    }

    pub fn pull(&self, remote: &str, local: &str) -> Result<()> {
        let _ = (remote, local);
        Ok(())
    }

    pub fn mkdir(&self, path: &str) -> Result<()> {
        let _ = path;
        Ok(())
    }

    pub fn delete(&self, path: &str, recursive: bool) -> Result<()> {
        let _ = (path, recursive);
        Ok(())
    }

    pub fn rename(&self, from: &str, to: &str) -> Result<()> {
        let _ = (from, to);
        Ok(())
    }
}
