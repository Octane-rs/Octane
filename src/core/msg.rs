use std::fmt;

use scrcpy_launcher::adb::server::DeviceLong;

use crate::core::primitives::async_state::AsyncResult;
use crate::services::adb::{AdbError, DeviceId};
use crate::services::session::{SessionConfig, SessionHandle};
use crate::ui::pages::CurrentPage;

/// Represents all possible inputs to the system.
pub enum Msg {
    // UI Events
    Navigate(CurrentPage),
    RequestAdbDevices,
    RequestStartSession(SessionConfig),
    RequestStopSession(DeviceId),
    ClearLogs,

    // System Events
    AdbDevicesLoaded(AsyncResult<Vec<DeviceLong>, AdbError>),
    SessionStarted {
        session: SessionHandle,
    },
    SessionStopped {
        device_id: DeviceId,
        error: Option<anyhow::Error>,
    },
}

impl fmt::Display for Msg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Navigate(_) => "Navigate",
            Self::RequestAdbDevices => "RequestAdbDevices",
            Self::RequestStartSession(_) => "RequestStartSession",
            Self::RequestStopSession(_) => "RequestStopSession",
            Self::ClearLogs => "ClearLogs",
            Self::AdbDevicesLoaded(_) => "AdbDevicesLoaded",
            Self::SessionStarted { .. } => "SessionStarted",
            Self::SessionStopped { .. } => "SessionStopped",
        };
        s.fmt(f)
    }
}
