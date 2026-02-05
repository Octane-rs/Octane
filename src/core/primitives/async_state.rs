//! Wrapper for loading asynchronous services data

use std::borrow::Cow;
use std::fmt::Display;
use std::mem;
use std::time::{Duration, Instant};

/// Represents the possible states of an asynchronous operation.
#[derive(Default, Debug, Clone)]
pub enum LoadState<T, E> {
    /// Default state, no ongoing operation.
    #[default]
    Idle,
    /// Operation is processing.
    Loading {
        /// Previously loaded data if any.
        prev: Option<T>,
        /// Loading start timestamp.
        time: Instant,
    },
    /// Operation successfully completed.
    Loaded(T),
    /// Operation failed.
    Error(E),
}

/// A standardized log message
pub enum Trace {
    Success(String),
    Error(String),
}

/// Generational unique identifier preventing stale results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ticket(u64);

/// Result of an asynchronous worker.
#[derive(Debug)]
pub struct AsyncResult<T, E> {
    /// Ticket issued when this specific task started.
    pub ticket: Ticket,
    /// Task result
    pub result: Result<T, E>,
}

/// Manages the state machine for asynchronous data loading.
///
/// - Tracking loading status.
/// - Measuring operation duration.
/// - Discarding stale results (race condition prevention).
/// - Retaining previous data during reloads.
pub struct AsyncState<T, E> {
    /// Data state
    state: LoadState<T, E>,
    /// Generation counter
    next_generation: u64,
    /// Ticket of the currently running operation.
    /// If `None`, no operation is valid.
    current_ticket: Option<Ticket>,
}

impl<T, E> AsyncState<T, E> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the inner state.
    pub const fn view(&self) -> &LoadState<T, E> {
        &self.state
    }

    /// Returns `true` if an operation is currently in progress.
    pub const fn is_loading(&self) -> bool {
        self.current_ticket.is_some()
    }

    /// Transitions the state to [`LoadState::Loading`] and issues a new ticket.
    ///
    /// This invalidates any previously running tasks by incrementing the internal
    /// generation counter. The returned [`Ticket`] must be passed to the worker task.
    ///
    /// # Returns
    ///
    /// - [`Ticket`] - A unique token that must be returned with the result.
    pub fn start_load(&mut self) -> Ticket {
        self.next_generation = self.next_generation.wrapping_add(1);
        let ticket = Ticket(self.next_generation);

        self.current_ticket = Some(ticket);

        let time = Instant::now();

        self.state = match mem::replace(&mut self.state, LoadState::Idle) {
            LoadState::Loading { prev, time: _ } => LoadState::Loading { prev, time },
            LoadState::Loaded(data) => LoadState::Loading {
                prev: Some(data),
                time,
            },
            LoadState::Idle | LoadState::Error(_) => LoadState::Loading { prev: None, time },
        };

        ticket
    }

    /// Applies a result to the state, enforcing ticket validation.
    ///
    /// If the `response.ticket` does not match the current expected ticket,
    /// the result is discarded and the state is unchanged.
    ///
    /// # Arguments
    ///
    /// - `result`: The async payload.
    ///
    /// # Returns
    ///
    /// - `Option<Duration>` - The time elapsed since `start_load` was called.
    ///   Returns `None` if the result was stale or invalid.
    pub fn apply(&mut self, result: AsyncResult<T, E>) -> Option<Duration> {
        if self.current_ticket != Some(result.ticket) {
            warn!("Dropped stale result (Gen: {})", result.ticket.0);
            return None;
        }

        self.current_ticket = None;

        let previous_state = mem::replace(&mut self.state, LoadState::Idle);

        if let LoadState::Loading { time, .. } = previous_state {
            let duration = time.elapsed();

            self.state = match result.result {
                Ok(data) => LoadState::Loaded(data),
                Err(err) => LoadState::Error(err),
            };

            Some(duration)
        } else {
            error!("Ticket match but state was not `Loading`. This is a logic bug.");
            None
        }
    }

    /// Applies the result and generates a standardized log report.
    ///
    /// # Arguments
    ///
    /// - `result`: The async payload.
    /// - `label`: Data label.
    /// - `summarize`: A closure to describe the success data.
    pub fn apply_trace<'a, F>(
        &'a mut self,
        result: AsyncResult<T, E>,
        label: &'a str,
        summarize: F,
    ) -> Option<Trace>
    where
        E: Display,
        F: FnOnce(&'a T) -> Cow<'a, str>,
    {
        let duration = self.apply(result)?;
        let seconds = duration.as_secs_f64();

        match self.view() {
            LoadState::Loaded(data) => {
                let summary = summarize(data);
                let msg = format!("{label} in {seconds:.2}s ({summary})");
                Some(Trace::Success(msg))
            }
            LoadState::Error(err) => {
                let msg = format!("{label} after {seconds:.2}s (Error: {err})");
                Some(Trace::Error(msg))
            }
            _ => None,
        }
    }
}

impl<T, E> Default for AsyncState<T, E> {
    fn default() -> Self {
        Self {
            state: LoadState::Idle,
            next_generation: 0,
            current_ticket: None,
        }
    }
}

impl<T, E> LoadState<T, E> {
    pub const fn value(&self) -> Option<&T> {
        match self {
            Self::Loading { prev, .. } => prev.as_ref(),
            Self::Loaded(value) => Some(value),
            _ => None,
        }
    }

    pub const fn error(&self) -> Option<&E> {
        match self {
            Self::Error(err) => Some(err),
            _ => None,
        }
    }
}

impl<T, E> AsyncResult<T, E> {
    /// Transforms the success value of the result, preserving the ticket.
    pub fn map<U>(self, op: impl FnOnce(T) -> U) -> AsyncResult<U, E> {
        AsyncResult {
            ticket: self.ticket,
            result: self.result.map(op),
        }
    }
}
