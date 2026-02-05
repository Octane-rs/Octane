use crate::core::primitives::async_state::Ticket;
use crate::services::session::{SessionConfig, SessionHandle};

#[derive(Debug)]
pub enum LogLevel {
    Trace,
    Info,
    Error,
}

pub enum Effect {
    /// Tell the generic UI loop to repaint
    Render,

    /// Log a message to stdout/stderr
    Log {
        level: LogLevel,
        msg: String,
    },

    /// Ask Shell to fetch devices
    FetchAdbDevices {
        ticket: Ticket,
    },

    /// Ask Shell to start a session
    StartSession {
        config: SessionConfig,
    },
    StopSession {
        session: SessionHandle,
    },
}
