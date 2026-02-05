use eframe::egui;
use eframe::egui::{
    Align, ComboBox, CursorIcon, Layout, RichText, Sense, Slider, Stroke, Ui, Vec2,
};

use crate::ui::pages::settings::state::{SessionSettings, VideoCodec};

pub struct VideoSection;
impl VideoSection {
    pub fn show(ui: &mut Ui, state: &mut SessionSettings) {
        card(ui, "Video Stream", |ui| {
            switch_row(ui, "Enable Video", &mut state.video_enabled);

            if state.video_enabled {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                combo_row(
                    ui,
                    "codec",
                    "Codec",
                    &mut state.codec,
                    vec![
                        (VideoCodec::H264, "H.264 (AVC)"),
                        (VideoCodec::H265, "H.265 (HEVC)"),
                        (VideoCodec::AV1, "AV1"),
                    ],
                );

                combo_row(
                    ui,
                    "res",
                    "Max Resolution",
                    &mut state.limit_resolution,
                    vec![
                        (0, "Native"),
                        (1920, "1920p"),
                        (1280, "1280p"),
                        (1080, "1080p"),
                        (720, "720p"),
                        (640, "640p"),
                        (480, "480p"),
                        (360, "360p"),
                    ],
                );

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Bitrate"));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(Slider::new(&mut state.bitrate_mbps, 1..=100).show_value(false));
                        ui.add_sized(
                            [60.0, 30.0],
                            egui::Label::new(format!("{} Mbps", state.bitrate_mbps)),
                        );
                    });
                });
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Frame Rate Limit"));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(Slider::new(&mut state.max_fps, 0..=120).show_value(false));
                        let text = if state.max_fps == 0 {
                            "Unlimited".to_string()
                        } else {
                            format!("{} fps", state.max_fps)
                        };
                        ui.add_sized([60.0, 30.0], egui::Label::new(text));
                    });
                });
                ui.add_space(4.0);

                switch_row(ui, "Hardware Decoding", &mut state.hw_decoder);
            }
        });
    }
}

pub struct AudioSection;
impl AudioSection {
    pub fn show(ui: &mut Ui, state: &mut SessionSettings) {
        card(ui, "Audio", |ui| {
            switch_row(
                ui,
                "Enable Audio Forwarding (Not implemented)",
                &mut state.audio_enabled,
            );

            if state.audio_enabled {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(RichText::new(
                    "Audio will be forwarded from the device to this computer.",
                ));
            }
        });
    }
}

pub struct ControlSection;
impl ControlSection {
    pub fn show(ui: &mut Ui, state: &mut SessionSettings) {
        card(ui, "Input Control", |ui| {
            switch_row(
                ui,
                "Enable Mouse & Keyboard (Not implemented)",
                &mut state.control_enabled,
            );

            if state.control_enabled {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(RichText::new(
                    "Mouse and keyboard events will be captured and sent to the device.",
                ));
            }
        });
    }
}

/// A rounded card container for a group of settings.
pub fn card(ui: &mut Ui, title: &str, content: impl FnOnce(&mut Ui)) {
    let frame = egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(12.0)
        .stroke(Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .inner_margin(16.0)
        .outer_margin(Vec2::new(0.0, 8.0));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(title)
                    .heading()
                    .color(ui.visuals().strong_text_color()),
            );
        });
        ui.add_space(12.0);

        content(ui);
    });
}

/// A row with a label on the left and a toggle/checkbox on the right.
pub fn switch_row(ui: &mut Ui, label: &str, value: &mut bool) {
    let (rect, mut response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::click());

    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        ui.painter().rect_filled(
            rect,
            2.0,
            ui.visuals().widgets.active.bg_fill.gamma_multiply(0.1),
        );
    }

    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }

    ui.horizontal(|ui| {
        ui.label(RichText::new(label));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.add(egui::Checkbox::new(value, "")).changed() {
                // *value = !*value;
            }
        });
    });
}

/// A row with a dropdown.
pub fn combo_row<T: PartialEq + Copy>(
    ui: &mut Ui,
    id_salt: &str,
    label: &str,
    current_value: &mut T,
    options: Vec<(T, &str)>,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let selected_text = options
                .iter()
                .find(|(val, _)| val == current_value)
                .map_or("?", |(_, text)| *text);

            ComboBox::from_id_salt(id_salt)
                .selected_text(selected_text)
                .width(100.0)
                .show_ui(ui, |ui| {
                    for (val, text) in options {
                        ui.selectable_value(current_value, val, text);
                    }
                });
        });
    });
    ui.add_space(4.0);
}
