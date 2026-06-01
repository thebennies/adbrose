use std::process::Command;

use tokio::process::Command as TokioCommand;

use crate::error::{AppError, Result};
use crate::file_entry::{Device, FileEntry, FileKind};

#[derive(Clone)]
pub struct AdbClient {
    pub serial: Option<String>,
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn parse_devices(output: &str) -> Vec<Device> {
    output
        .lines()
        .skip_while(|line| !line.contains("List of devices"))
        .skip(1)
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let parts: Vec<&str> = line.splitn(2, |c: char| c == '\t' || c == ' ').collect();
            if parts.len() == 2 {
                Some(Device {
                    serial: parts[0].to_string(),
                    state: parts[1].to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

fn parse_ls_output(output: &str) -> Vec<FileEntry> {
    output
        .lines()
        .filter(|l| !l.is_empty())
        .filter(|line| line.starts_with('d') || line.starts_with('-') || line.starts_with('l'))
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 7 {
                return None;
            }
            let kind = match line.chars().next() {
                Some('d') => FileKind::Directory,
                Some('l') => FileKind::Symlink,
                _ => FileKind::File,
            };
            let size: u64 = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
            let date_part = parts.get(5).unwrap_or(&"");
            let time_part = parts.get(6).unwrap_or(&"");
            let modified = format!("{} {}", date_part, time_part);
            let raw_name = if parts.len() > 8 {
                parts[7..].join(" ")
            } else {
                parts.get(7).unwrap_or(&"").to_string()
            };
            // Strip symlink target ("name -> target")
            let name = if kind == FileKind::Symlink {
                raw_name.split(" -> ").next().unwrap_or(&raw_name).to_string()
            } else {
                raw_name
            };
            if name == "." || name == ".." {
                return None;
            }
            Some(FileEntry {
                name,
                kind,
                size,
                modified: Some(modified),
            })
        })
        .collect()
}

impl AdbClient {
    pub fn new(serial: Option<String>) -> Result<Self> {
        Self::check_adb_exists()?;
        Ok(Self { serial })
    }

    fn check_adb_exists() -> Result<()> {
        if let Ok(path) = std::env::var("PATH") {
            for dir in path.split(':') {
                let full = std::path::Path::new(dir).join("adb");
                if full.exists() {
                    return Ok(());
                }
            }
        }
        Err(AppError::AdbNotFound)
    }

    fn base_cmd(&self) -> Command {
        let mut cmd = Command::new("adb");
        if let Some(ref serial) = self.serial {
            cmd.arg("-s").arg(serial);
        }
        cmd
    }

    fn run(&self, args: &[&str]) -> Result<String> {
        let output = self.base_cmd().args(args).output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("unauthorized") {
                return Err(AppError::Unauthorized);
            }
            if stderr.contains("device not found") || stderr.contains("no devices") {
                return Err(AppError::NoDevice);
            }
            return Err(AppError::Adb(stderr.to_string()));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn run_shell(&self, args: &[&str]) -> Result<String> {
        let mut all_args = vec!["shell"];
        all_args.extend_from_slice(args);
        self.run(&all_args)
    }

    pub fn device_list(&self) -> Result<Vec<Device>> {
        let output = self.run(&["devices"])?;
        Ok(parse_devices(&output))
    }

    pub fn list_dir(&self, path: &str) -> Result<Vec<FileEntry>> {
        let quoted = shell_quote(path);

        // Use ls -la as the primary method — it works on all Android devices
        // (toybox/busybox). The previous find -printf approach relied on a GNU
        // extension not available on Android, causing empty directory listings.
        let ls_output = self.run_shell(&["ls", "-la", &quoted])?;
        let mut entries = parse_ls_output(&ls_output);

        // Always add ../ so the user can navigate to the parent folder with Enter
        entries.push(FileEntry {
            name: "../".into(),
            kind: FileKind::Directory,
            size: 0,
            modified: None,
        });

        entries.sort();
        Ok(entries)
    }

    #[allow(dead_code)]
    pub fn push(&self, local: &str, remote: &str) -> Result<()> {
        let output = self.run(&["push", local, remote])?;
        if output.contains("error") {
            return Err(AppError::Adb(output));
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn pull(&self, remote: &str, local: &str) -> Result<()> {
        let output = self.run(&["pull", remote, local])?;
        if output.contains("error") {
            return Err(AppError::Adb(output));
        }
        Ok(())
    }

    pub fn mkdir(&self, path: &str) -> Result<()> {
        let quoted = shell_quote(path);
        self.run_shell(&["mkdir", "-p", &quoted])?;
        Ok(())
    }

    pub fn delete(&self, path: &str, recursive: bool) -> Result<()> {
        let quoted = shell_quote(path);
        if recursive {
            self.run_shell(&["rm", "-rf", &quoted])?;
        } else {
            self.run_shell(&["rm", "-f", &quoted])?;
        }
        Ok(())
    }

    pub fn rename(&self, from: &str, to: &str) -> Result<()> {
        let quoted_from = shell_quote(from);
        let quoted_to = shell_quote(to);
        self.run_shell(&["mv", &quoted_from, &quoted_to])?;
        Ok(())
    }

    fn base_async_cmd(&self) -> TokioCommand {
        let mut cmd = TokioCommand::new("adb");
        if let Some(ref serial) = self.serial {
            cmd.arg("-s").arg(serial);
        }
        cmd
    }

    pub async fn push_with_progress<F: Fn(u8) + Send>(
        &self,
        local: &str,
        remote: &str,
        on_progress: F,
    ) -> Result<()> {
        let mut child = self.base_async_cmd()
            .args(["push", local, remote])
            .stderr(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        if let Some(stderr) = child.stderr.take() {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(percent) = parse_progress(&line) {
                    on_progress(percent);
                }
            }
        }

        let status = child.wait().await?;
        if !status.success() {
            return Err(AppError::Adb("push failed".into()));
        }
        Ok(())
    }

    pub async fn pull_with_progress<F: Fn(u8) + Send>(
        &self,
        remote: &str,
        local: &str,
        on_progress: F,
    ) -> Result<()> {
        let mut child = self.base_async_cmd()
            .args(["pull", remote, local])
            .stderr(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        if let Some(stderr) = child.stderr.take() {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(percent) = parse_progress(&line) {
                    on_progress(percent);
                }
            }
        }

        let status = child.wait().await?;
        if !status.success() {
            return Err(AppError::Adb("pull failed".into()));
        }
        Ok(())
    }

    pub async fn tar_pull(
        &self,
        remote_dir: &str,
        local_dir: &str,
    ) -> Result<()> {
        let remote_path = std::path::Path::new(remote_dir);
        let remote_parent = remote_path.parent().and_then(|p| p.to_str()).unwrap_or(".");
        let remote_base = remote_path.file_name().and_then(|n| n.to_str()).unwrap_or(remote_dir);

        // Use sh -c so shell_quote works correctly, and cd to the parent so the
        // tar archive contains just the base folder name (not the full path).
        let shell_cmd = format!("cd {} && tar cf - {}", shell_quote(remote_parent), shell_quote(remote_base));
        let mut adb_child = self.base_async_cmd()
            .args(["exec-out", "sh", "-c", &shell_cmd])
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        let adb_stdout = adb_child.stdout.take()
            .ok_or_else(|| AppError::Adb("failed to capture adb stdout".into()))?;

        let stdin_pipe: std::process::Stdio = adb_stdout
            .try_into()
            .map_err(|_| AppError::Adb("failed to convert stdout to stdio".into()))?;
        let tar_status = tokio::process::Command::new("tar")
            .args(["xf", "-"])
            .current_dir(local_dir)
            .stdin(stdin_pipe)
            .status()
            .await?;

        let adb_status = adb_child.wait().await?;

        if !tar_status.success() || !adb_status.success() {
            return Err(AppError::Adb("tar streaming pull failed".into()));
        }
        Ok(())
    }
}

fn parse_progress(line: &str) -> Option<u8> {
    let line = line.trim();
    if !line.starts_with('[') {
        return None;
    }
    let end = line.find(']')?;
    let inner = &line[1..end];
    let num_str = inner.trim().trim_end_matches('%');
    num_str.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_simple() {
        assert_eq!(shell_quote("/sdcard"), "'/sdcard'");
    }

    #[test]
    fn shell_quote_spaces() {
        assert_eq!(shell_quote("/sdcard/My Documents"), "'/sdcard/My Documents'");
    }

    #[test]
    fn shell_quote_single_quote() {
        assert_eq!(shell_quote("/sdcard/it's"), "'/sdcard/it'\\''s'");
    }

    #[test]
    fn shell_quote_special_chars() {
        assert_eq!(shell_quote("/sdcard/$foo`bar\"baz\""), "'/sdcard/$foo`bar\"baz\"'");
    }

    #[test]
    fn parse_devices_empty() {
        let devices = parse_devices("");
        assert!(devices.is_empty());
    }

    #[test]
    fn parse_devices_typical() {
        let output = "List of devices attached\nabc123\tdevice\nxyz789\tunauthorized\n";
        let devices = parse_devices(output);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].serial, "abc123");
        assert_eq!(devices[0].state, "device");
        assert_eq!(devices[1].serial, "xyz789");
        assert_eq!(devices[1].state, "unauthorized");
    }

    #[test]
    fn parse_devices_no_device() {
        let output = "List of devices attached\n";
        let devices = parse_devices(output);
        assert!(devices.is_empty());
    }

    #[test]
    fn parse_ls_output_basic() {
        let output = "drwxrwx--x 4 root sdcard_rw 4096 2024-01-15 10:30 DCIM\n-rw-rw---- 1 root sdcard_rw 2048 2024-01-15 10:30 file.txt\n";
        let entries = parse_ls_output(output);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "DCIM");
        assert!(entries[0].is_dir());
        assert_eq!(entries[1].name, "file.txt");
        assert_eq!(entries[1].size, 2048);
    }

    #[test]
    fn parse_progress_line() {
        assert_eq!(parse_progress("[  0%] /data/local/tmp/file.txt"), Some(0));
        assert_eq!(parse_progress("[ 47%] /data/local/tmp/file.txt"), Some(47));
        assert_eq!(parse_progress("[100%] /data/local/tmp/file.txt"), Some(100));
        assert_eq!(parse_progress("some other output"), None);
        assert_eq!(parse_progress("[50%]"), Some(50));
    }

    #[test]
    fn parse_ls_output_empty() {
        let entries = parse_ls_output("");
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_ls_output_symlink() {
        let output = "lrwxrwxrwx 1 root root 11 2024-01-15 10:30 sdcard -> /storage/self/primary\n";
        let entries = parse_ls_output(output);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "sdcard");
        assert!(matches!(entries[0].kind, FileKind::Symlink));
    }
}
