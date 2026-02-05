//! The functional core of the application.
//!
//! This module contains the pure business logic, application state ([`model`]),
//! and intent definitions ([`msg`], [`effect`]).
//!
//! # Important
//!
//! No side effects (IO, spawning threads, WGPU, etc.) are allowed here.
//! All changes must be returned as `Effect` enums to be executed by the Shell.
//!

pub mod effect;
pub mod logs;
pub mod model;
pub mod msg;
pub mod primitives;
