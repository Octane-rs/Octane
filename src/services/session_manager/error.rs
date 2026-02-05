#[derive(Debug, Error, Clone)]
pub enum SessionManagerError {
    /// The connection to the Session manager service was lost.
    #[error("Internal service disconnected")]
    ChannelClosed,
}

impl SessionManagerError {
    /// User friendly error title
    pub const fn title(&self) -> &'static str {
        match self {
            Self::ChannelClosed => "System Error",
        }
    }
}
