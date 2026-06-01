use crate::error::Result;
use crate::file_entry::FileEntry;
use std::path::Path;

pub fn list_dir(path: &Path) -> Result<Vec<FileEntry>> {
    let _ = path;
    Ok(vec![])
}

pub fn mkdir(path: &Path) -> Result<()> {
    let _ = path;
    Ok(())
}

pub fn delete(path: &Path, recursive: bool) -> Result<()> {
    let _ = (path, recursive);
    Ok(())
}

pub fn rename(from: &Path, to: &Path) -> Result<()> {
    let _ = (from, to);
    Ok(())
}
