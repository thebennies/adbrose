use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub transfer: TransferConfig,
    #[serde(default)]
    pub bookmarks: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultsConfig {
    #[serde(default)]
    pub android_path: Option<String>,
    #[serde(default)]
    pub local_path: Option<String>,
    #[serde(default)]
    pub serial: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransferConfig {
    #[serde(default = "default_true")]
    pub use_tar_streaming: bool,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            android_path: None,
            local_path: None,
            serial: None,
        }
    }
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            use_tar_streaming: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            defaults: DefaultsConfig::default(),
            transfer: TransferConfig::default(),
            bookmarks: HashMap::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("abdrose")
        .join("config.toml")
}

pub fn load() -> Config {
    let path = config_path();
    if !path.exists() {
        return Config::default();
    }
    match fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save(config: &Config) -> std::io::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config).unwrap_or_default();
    fs::write(&path, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = Config::default();
        assert!(config.defaults.android_path.is_none());
        assert!(config.transfer.use_tar_streaming);
        assert!(config.bookmarks.is_empty());
    }

    #[test]
    fn roundtrip_toml() {
        let mut config = Config::default();
        config.defaults.android_path = Some("/sdcard/Download".into());
        config.bookmarks.insert("work".into(), "/sdcard/Download".into());
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.defaults.android_path.as_deref(), Some("/sdcard/Download"));
        assert_eq!(parsed.bookmarks.get("work").unwrap(), "/sdcard/Download");
        assert!(parsed.transfer.use_tar_streaming);
    }

    #[test]
    fn parse_empty_toml() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.defaults.android_path.is_none());
        assert!(config.transfer.use_tar_streaming);
    }

    #[test]
    fn parse_full_toml() {
        let toml_str = r#"
[defaults]
android_path = "/sdcard/DCIM"
serial = "abc123"

[transfer]
use_tar_streaming = false

[bookmarks]
photos = "/sdcard/DCIM"
home = "~"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.defaults.android_path.as_deref(), Some("/sdcard/DCIM"));
        assert_eq!(config.defaults.serial.as_deref(), Some("abc123"));
        assert!(!config.transfer.use_tar_streaming);
        assert_eq!(config.bookmarks.len(), 2);
        assert_eq!(config.bookmarks.get("photos").unwrap(), "/sdcard/DCIM");
    }

    #[test]
    fn save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut config = Config::default();
        config.bookmarks.insert("test".into(), "/tmp".into());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, toml::to_string_pretty(&config).unwrap()).unwrap();
        let loaded: Config = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.bookmarks.get("test").unwrap(), "/tmp");
    }
}
