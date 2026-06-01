use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum FileKind {
    Directory,
    File,
    Symlink,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileEntry {
    pub name: String,
    pub kind: FileKind,
    pub size: u64,
    pub modified: Option<String>,
}

impl FileEntry {
    pub fn is_dir(&self) -> bool {
        self.kind == FileKind::Directory
    }
}

impl Eq for FileEntry {}

impl Ord for FileEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.is_dir(), other.is_dir()) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => self.name.to_lowercase().cmp(&other.name.to_lowercase()),
        }
    }
}

impl PartialOrd for FileEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct Device {
    pub serial: String,
    pub state: String,
}

pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if size >= GB {
        format!("{:.1}G", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1}M", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1}K", size as f64 / KB as f64)
    } else {
        format!("{}B", size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directories_sort_before_files() {
        let dir = FileEntry {
            name: "zzz".into(),
            kind: FileKind::Directory,
            size: 0,
            modified: None,
        };
        let file = FileEntry {
            name: "aaa".into(),
            kind: FileKind::File,
            size: 10,
            modified: None,
        };
        assert!(dir < file);
    }

    #[test]
    fn format_size_units() {
        assert_eq!(format_size(0), "0B");
        assert_eq!(format_size(500), "500B");
        assert_eq!(format_size(1024), "1.0K");
        assert_eq!(format_size(1048576), "1.0M");
        assert_eq!(format_size(1073741824), "1.0G");
    }

    #[test]
    fn case_insensitive_sort() {
        let a = FileEntry {
            name: "Banana".into(),
            kind: FileKind::File,
            size: 0,
            modified: None,
        };
        let b = FileEntry {
            name: "apple".into(),
            kind: FileKind::File,
            size: 0,
            modified: None,
        };
        assert!(b < a);
    }
}
