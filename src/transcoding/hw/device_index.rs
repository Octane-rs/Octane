/// Physical hardware device identifier
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HWDeviceIndex {
    /// Global device
    Global,
    /// Device with a specific index
    Index(u8),
}

impl From<u8> for HWDeviceIndex {
    fn from(index: u8) -> Self {
        Self::Index(index)
    }
}
