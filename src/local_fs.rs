use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::error::{AppError, Result};
use crate::file_entry::{FileEntry, FileKind};

pub fn list_dir(path: &Path) -> Result<Vec<FileEntry>> {
    let mut entries: Vec<FileEntry> = Vec::new();

    // Always add ../ so the user can navigate to the parent folder with Enter
    entries.push(FileEntry {
        name: "../".into(),
        kind: FileKind::Directory,
        size: 0,
        modified: None,
    });

    let read_dir = fs::read_dir(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::NotFound(path.display().to_string())
        } else if e.kind() == std::io::ErrorKind::PermissionDenied {
            AppError::PermissionDenied(path.display().to_string())
        } else {
            AppError::Io(e)
        }
    })?;

    for entry in read_dir {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let name = entry.file_name().to_string_lossy().to_string();
        let kind = if metadata.is_dir() {
            FileKind::Directory
        } else if metadata.is_symlink() {
            FileKind::Symlink
        } else {
            FileKind::File
        };
        let size = metadata.len();
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| {
                let secs = d.as_secs();
                let days = secs / 86400;
                let hours = (secs % 86400) / 3600;
                let mins = (secs % 3600) / 60;
                format!("{}d {:02}:{:02}", days, hours, mins)
            });
        entries.push(FileEntry {
            name,
            kind,
            size,
            modified,
        });
    }

    entries.sort();
    Ok(entries)
}

pub fn mkdir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn delete(path: &Path, recursive: bool) -> Result<()> {
    if path.is_dir() && recursive {
        fs::remove_dir_all(path)?;
    } else if path.is_dir() {
        fs::remove_dir(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn rename(from: &Path, to: &Path) -> Result<()> {
    fs::rename(from, to)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn list_dir_reads_entries() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("file.txt"), "hello").unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();

        let entries = list_dir(dir.path()).unwrap();
        assert!(entries.iter().any(|e| e.name == "file.txt" && !e.is_dir()));
        assert!(entries.iter().any(|e| e.name == "subdir" && e.is_dir()));
    }

    #[test]
    fn list_dir_sorts_dirs_first() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("aaa.txt"), "x").unwrap();
        fs::create_dir(dir.path().join("zzz_dir")).unwrap();

        let entries = list_dir(dir.path()).unwrap();
        assert_eq!(entries[0].name, "../");
        assert!(entries[0].is_dir());
        assert_eq!(entries[1].name, "zzz_dir");
        assert!(entries[1].is_dir());
    }

    #[test]
    fn mkdir_creates_directory() {
        let dir = tempfile::tempdir().unwrap();
        let new_dir = dir.path().join("new_folder");
        mkdir(&new_dir).unwrap();
        assert!(new_dir.is_dir());
    }

    #[test]
    fn delete_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("deleteme.txt");
        fs::write(&file, "bye").unwrap();
        delete(&file, false).unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn delete_recursive_removes_dir() {
        let dir = tempfile::tempdir().unwrap();
        let subdir = dir.path().join("sub");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("file.txt"), "x").unwrap();
        delete(&dir.path().join("sub"), true).unwrap();
        assert!(!subdir.exists());
    }

    #[test]
    fn rename_changes_name() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("old.txt");
        let to = dir.path().join("new.txt");
        fs::write(&from, "data").unwrap();
        rename(&from, &to).unwrap();
        assert!(to.exists());
        assert!(!from.exists());
    }
}
