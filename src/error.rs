use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("adb not found in PATH – install with: brew install android-platform-tools")]
    AdbNotFound,
    #[error("no Android device connected")]
    NoDevice,
    #[error("device unauthorized – check your phone and accept the USB debugging prompt")]
    Unauthorized,
    #[error("adb error: {0}")]
    Adb(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("path not found: {0}")]
    NotFound(String),
    #[error("transfer failed: {0}")]
    TransferFailed(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
