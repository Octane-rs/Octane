use std::collections::VecDeque;

use chrono::{DateTime, Utc};

use crate::core::primitives::async_state::Trace;

/// Defines a log entry level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Error,
    Success,
}

/// A fixed-capacity FILO ring buffer for storing application logs.
pub struct LogStore {
    entries: VecDeque<LogEntry>,
    capacity: usize,
}

/// A single record within the log store.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub level: LogLevel,
}

impl LogStore {
    /// Creates a new log store with the specified maximum capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Logs an informational message.
    pub fn info(&mut self, message: impl Into<String>) {
        self.push(message, LogLevel::Info);
    }

    /// Logs an error message.
    pub fn error(&mut self, message: impl Into<String>) {
        self.push(message, LogLevel::Error);
    }

    /// Logs a success message.
    pub fn success(&mut self, message: impl Into<String>) {
        self.push(message, LogLevel::Success);
    }

    /// Logs an [`AsyncState`] trace result.
    ///
    /// - If the trace is `Some(Success)`, it logs a success message.
    /// - If the trace is `Some(Error)`, it logs an error message.
    /// - If `None`, no action is taken.
    pub fn trace(&mut self, trace: Option<Trace>) {
        match trace {
            Some(Trace::Success(msg)) => self.success(msg),
            Some(Trace::Error(msg)) => self.error(msg),
            None => {}
        }
    }

    /// Internal helper to push a message and enforce capacity.
    fn push(&mut self, message: impl Into<String>, level: LogLevel) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(LogEntry {
            timestamp: Utc::now(),
            message: message.into(),
            level,
        });
    }

    /// Returns an iterator over the log entries.
    pub fn iter(&self) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter()
    }

    /// Removes all log entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
