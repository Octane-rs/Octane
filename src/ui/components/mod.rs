//! Reusable widgets

use eframe::egui::{Response, Ui};

use crate::ui::context::ViewContext;

pub mod common;
pub mod features;
pub mod layouts;
mod style;

/// A widget accessing to the application context
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub trait CtxWidget {
    /// Allocate space, interact, paint, and return a [`Response`].
    fn ui(self, ui: &mut Ui, ctx: &mut ViewContext<'_>) -> Response;
}
