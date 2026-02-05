use scrcpy_launcher::RustADBError;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum AdbError {
    /// The connection to the ADB actor was lost.
    #[error("Internal service disconnected")]
    ChannelClosed,
    /// An error occurred within the underlying ADB client library or server.
    #[error(transparent)]
    Adb(#[from] RustADBError),
    /// A tokio task panicked or was canceled
    #[error("Task failed: {0}")]
    Join(#[from] JoinError),
}

impl AdbError {
    /// User friendly error title
    pub const fn title(&self) -> &'static str {
        match self {
            Self::ChannelClosed => "System Error",
            Self::Adb(_) => "ADB Error",
            Self::Join(_) => "Crash Report",
        }
    }
}
