//! Main application pages

use std::fmt;

use eframe::egui::Ui;

use crate::ui::context::ViewContext;

pub mod home;
pub mod settings;

pub trait Page {
    /// Called each time the UI needs repainting, which may be many times per second
    fn show(&mut self, ui: &mut Ui, ctx: &mut ViewContext<'_>);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentPage {
    Home,
    Settings,
}

impl fmt::Display for CurrentPage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Home => "home",
            Self::Settings => "settings",
        };
        s.fmt(f)
    }
}
