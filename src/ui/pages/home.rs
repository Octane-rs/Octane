use eframe::egui::{RichText, TextStyle, Ui, Widget, vec2};

use crate::core::primitives::async_state::LoadState;
use crate::ui::components::CtxWidget;
use crate::ui::components::features::device_list::DeviceList;
use crate::ui::components::features::log_view::LogView;
use crate::ui::context::ViewContext;
use crate::ui::pages::Page;

#[derive(Default)]
pub struct HomePage;

impl HomePage {
    pub const fn new() -> Self {
        Self
    }
}

impl Page for HomePage {
    fn show(&mut self, ui: &mut Ui, ctx: &mut ViewContext<'_>) {
        let available_height = ui.available_height();
        let top_half_height = available_height * 0.5;

        ui.allocate_ui(vec2(ui.available_width(), top_half_height), |ui| {
            let adb_devices = ctx.model.adb_devices.view();
            match adb_devices {
                LoadState::Idle
                | LoadState::Loading {
                    prev: None,
                    time: _,
                } => {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new("Loading ADB devices ...")
                                .text_style(TextStyle::Body)
                                .color(ui.visuals().weak_text_color()),
                        );
                    });
                }

                LoadState::Loading {
                    prev: Some(devices),
                    ..
                }
                | LoadState::Loaded(devices) => {
                    DeviceList::new(devices).ui(ui, ctx);
                }

                LoadState::Error(err) => {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new(err.to_string())
                                .text_style(TextStyle::Body)
                                .color(ui.visuals().weak_text_color()),
                        );
                    });
                }
            }
        });

        ui.allocate_ui(vec2(ui.available_width(), top_half_height), |ui| {
            LogView::new(&ctx.model.logs).ui(ui);
        });
    }
}
