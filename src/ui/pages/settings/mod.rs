use eframe::egui::{self, Align, Button, Layout, RichText, ScrollArea, Ui, Vec2};

use crate::core::msg::Msg;
use crate::ui::context::ViewContext;
use crate::ui::pages::settings::components::{AudioSection, ControlSection, VideoSection};
pub use crate::ui::pages::settings::state::SessionSettings;
use crate::ui::pages::{CurrentPage, Page};

mod components;
mod state;

pub struct SettingsPage;

impl SettingsPage {
    pub const fn new() -> Self {
        Self
    }

    fn render_header(&self, ui: &mut Ui, ctx: &mut ViewContext<'_>) {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Session Configuration").size(24.0).strong());

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add_sized(
                        Vec2::new(60.0, 30.0),
                        Button::new("Reset").corner_radius(5.0),
                    )
                    .clicked()
                {
                    let data = &mut *ctx.data;

                    data.session_settings = SessionSettings::default();
                }
            });
        });
        ui.add_space(20.0);
    }

    fn render_footer(&self, ui: &mut Ui, ctx: &mut ViewContext<'_>) {
        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);

        ui.take_available_width();

        let exit_btn = ui.add_sized(
            Vec2::new(ui.available_width(), 48.0),
            Button::new(RichText::new("Validate").strong()).corner_radius(8.0),
        );

        if exit_btn.clicked() {
            ctx.send(Msg::Navigate(CurrentPage::Home));
        }
    }
}

impl Page for SettingsPage {
    fn show(&mut self, ui: &mut Ui, ctx: &mut ViewContext<'_>) {
        egui::CentralPanel::default().show(ui.ctx(), |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.set_max_width(ui.available_width().min(800.0));

                        egui::Frame::new()
                            .inner_margin(egui::Margin::symmetric(30, 0))
                            .show(ui, |ui| {
                                ui.add_space(16.0);

                                self.render_header(ui, ctx);

                                VideoSection::show(ui, &mut ctx.data.session_settings);
                                AudioSection::show(ui, &mut ctx.data.session_settings);
                                ControlSection::show(ui, &mut ctx.data.session_settings);

                                self.render_footer(ui, ctx);

                                ui.add_space(30.0);
                            });
                    });
                });
        });
    }
}
