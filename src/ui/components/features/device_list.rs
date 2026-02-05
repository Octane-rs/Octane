use eframe::egui::{
    Align, Button, Color32, CursorIcon, Frame, Grid, Layout, Response, RichText, ScrollArea, Sense,
    TextStyle, Ui, Vec2, ViewportId, vec2,
};
use scrcpy_launcher::adb::server::{DeviceLong, DeviceState};

use crate::core::msg::Msg;
use crate::ui::components::CtxWidget;
use crate::ui::context::ViewContext;
use crate::ui::pages::CurrentPage;

pub struct DeviceList<'a> {
    devices: &'a [DeviceLong],
}

impl<'a> DeviceList<'a> {
    pub const fn new(devices: &'a [DeviceLong]) -> Self {
        Self { devices }
    }
}

impl CtxWidget for DeviceList<'_> {
    fn ui(self, ui: &mut Ui, ctx: &mut ViewContext<'_>) -> Response {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Connected Devices");

                if !self.devices.is_empty() {
                    ui.heading("•");
                    ui.heading(self.devices.len().to_string());
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add_sized(
                            Vec2::new(70.0, 30.0),
                            Button::new("Settings").corner_radius(5.0),
                        )
                        .clicked()
                    {
                        ctx.send(Msg::Navigate(CurrentPage::Settings));
                    }
                });
            });

            ui.separator();

            if self.devices.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("No devices found via ADB.")
                            .text_style(TextStyle::Body)
                            .color(ui.visuals().weak_text_color()),
                    );
                });
            } else {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.take_available_width();

                        for device in self.devices {
                            ui.add_space(5.0);
                            Device::new(device).ui(ui, ctx);
                        }
                    });
            }
        });

        ui.response()
    }
}

struct Device<'a> {
    device: &'a DeviceLong,
}

impl<'a> Device<'a> {
    pub const fn new(device: &'a DeviceLong) -> Self {
        Self { device }
    }
}

impl CtxWidget for Device<'_> {
    fn ui(self, ui: &mut Ui, ctx: &mut ViewContext<'_>) -> Response {
        let (state_label, state_color, icon, can_connect) = match self.device.state {
            DeviceState::Device => ("Online", Color32::GREEN, "📱", true),
            DeviceState::Authorizing => ("Authorizing...", Color32::YELLOW, "⏳", false),
            DeviceState::Connecting => ("Connecting...", Color32::YELLOW, "🔄", false),
            DeviceState::Unauthorized => ("Unauthorized", Color32::RED, "🔒", false),
            DeviceState::NoPerm => ("No Permission", Color32::RED, "⛔", false),
            DeviceState::Bootloader => ("Fastboot", Color32::LIGHT_BLUE, "🔧", false),
            DeviceState::Recovery => ("Recovery", Color32::LIGHT_BLUE, "🚑", false),
            DeviceState::Sideload => ("Sideload", Color32::LIGHT_BLUE, "📦", false),
            DeviceState::Rescue => ("Rescue", Color32::RED, "🆘", false),
            DeviceState::Offline => ("Offline", Color32::GRAY, "🔌", false),
            DeviceState::Detached => ("Detached", Color32::GRAY, "🔌", false),
            DeviceState::NoDevice => ("No Device", Color32::RED, "📵", false),
            DeviceState::Host => ("Host", Color32::GRAY, "💻", false),
        };

        let frame_response = Frame::group(ui.style())
            .inner_margin(10.0)
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.take_available_width();

                    ui.horizontal_centered(|ui| {
                        ui.label(RichText::new(icon).size(24.0));
                    });

                    ui.add_space(8.0);

                    ui.vertical(|ui| {
                        let name = if self.device.model.is_empty() {
                            self.device.identifier.clone()
                        } else {
                            format!("{} ({})", self.device.model, self.device.product)
                        };

                        ui.label(RichText::new(name).strong().text_style(TextStyle::Body));

                        ui.horizontal(|ui| {
                            let (rect, _) = ui.allocate_exact_size(vec2(6.0, 6.0), Sense::hover());
                            ui.painter().circle_filled(rect.center(), 3.0, state_color);

                            ui.label(
                                RichText::new(state_label)
                                    .color(state_color)
                                    .text_style(TextStyle::Small),
                            );

                            ui.label(
                                RichText::new("•")
                                    .color(ui.visuals().weak_text_color())
                                    .text_style(TextStyle::Small),
                            );

                            ui.label(
                                RichText::new(&self.device.identifier)
                                    .monospace()
                                    .color(ui.visuals().weak_text_color())
                                    .text_style(TextStyle::Small),
                            );

                            ui.label(
                                RichText::new("•")
                                    .color(ui.visuals().weak_text_color())
                                    .text_style(TextStyle::Small),
                            );

                            ui.label(
                                RichText::new(&self.device.usb)
                                    .monospace()
                                    .color(ui.visuals().weak_text_color())
                                    .text_style(TextStyle::Small),
                            );
                        });
                    });
                });
            })
            .response;

        let interaction = ui.interact(frame_response.rect, frame_response.id, Sense::click());

        let interaction = if can_connect {
            let interaction = interaction.on_hover_cursor(CursorIcon::PointingHand);

            if interaction.clicked() {
                let context = ui.ctx().clone();
                let viewport_id = ViewportId::from_hash_of(self.device.identifier.as_str());

                ctx.send(Msg::RequestStartSession(
                    ctx.data.session_settings.to_config(
                        self.device.identifier.clone(),
                        Box::new(move || {
                            context.request_repaint_of(viewport_id);
                        }),
                    ),
                ));
            }

            if interaction.hovered() {
                ui.painter().rect_filled(
                    frame_response.rect,
                    6.0,
                    ui.visuals().widgets.hovered.bg_fill.gamma_multiply(0.2),
                );
            }

            interaction
        } else {
            interaction.on_hover_cursor(CursorIcon::NotAllowed)
        };

        interaction.on_hover_ui_at_pointer(|ui| {
            ui.heading("Device Details");
            ui.separator();
            Grid::new("device_details_tooltip")
                .num_columns(2)
                .spacing([15.0, 4.0])
                .show(ui, |ui| {
                    let mut row = |label: &str, value: &str| {
                        ui.label(RichText::new(label));
                        ui.monospace(value);
                        ui.end_row();
                    };

                    row("Serial:", &self.device.identifier);
                    row("Transport ID:", &self.device.transport_id.to_string());
                    row("Model:", &self.device.model);
                    row("Product:", &self.device.product);
                    row("Device:", &self.device.device);
                    row("USB Port:", &self.device.usb);
                });
        });

        frame_response
    }
}
