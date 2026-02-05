//! The imperative shell and runtime environment.
//!
//! This module manages the application lifecycle, the [`tokio`] runtime, window creation,
//! and the execution of side effects via [`capabilities`].
//!
//! It bridges the core Effects and translate them into concrete action.

pub mod app;
mod capabilities;
mod pollers;
