//! The presentation layer and UI components.
//!
//! This module contains the [`egui`] view logic, [`pages`], and [`views`].
//!
//! Views here are mostly pure functions that take the [`context`]
//! to render the interface and emit messages, without mutating the state.

mod components;
pub mod context;
pub mod pages;
pub mod perf;
pub mod renderer;
pub mod views;
