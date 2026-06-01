use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "adbrose", about = "Dual-pane TUI file browser for local and Android filesystems")]
pub struct Cli {
    #[arg(long, help = "Starting Android path [default: /sdcard]")]
    pub android_path: Option<String>,

    #[arg(long, help = "Starting local path [default: current directory]")]
    pub local_path: Option<String>,

    #[arg(long, help = "ADB device serial number")]
    pub serial: Option<String>,

    #[arg(long, help = "Log file path")]
    pub log_file: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_defaults() {
        let cli = Cli::try_parse_from(["adbrose"]).unwrap();
        assert!(cli.android_path.is_none());
        assert!(cli.local_path.is_none());
        assert!(cli.serial.is_none());
        assert!(cli.log_file.is_none());
    }

    #[test]
    fn parse_all_args() {
        let cli = Cli::try_parse_from([
            "adbrose",
            "--android-path", "/sdcard/Download",
            "--local-path", "/home/user",
            "--serial", "abc123",
            "--log-file", "/tmp/adbrose.log",
        ]).unwrap();
        assert_eq!(cli.android_path.as_deref(), Some("/sdcard/Download"));
        assert_eq!(cli.local_path.as_deref(), Some("/home/user"));
        assert_eq!(cli.serial.as_deref(), Some("abc123"));
        assert_eq!(cli.log_file.as_deref(), Some("/tmp/adbrose.log"));
    }
}
