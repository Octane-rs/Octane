use eframe::egui;
use eframe::egui::{Response, RichText, ScrollArea, TextStyle, Ui, Widget};

use crate::core::logs::{LogEntry, LogLevel, LogStore};

pub struct LogView<'a> {
    store: &'a LogStore,
}

impl<'a> LogView<'a> {
    pub const fn new(store: &'a LogStore) -> Self {
        Self { store }
    }
}

impl Widget for LogView<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        egui::Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(8.0)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Logs").strong());
                    ui.separator();

                    ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.take_available_width();

                            for (index, entry) in self.store.iter().enumerate() {
                                LogRow::new(index, entry).ui(ui);
                            }
                        });
                });
            })
            .response
    }
}

pub struct LogRow<'a> {
    index: usize,
    entry: &'a LogEntry,
}

impl<'a> LogRow<'a> {
    pub const fn new(index: usize, entry: &'a LogEntry) -> Self {
        Self { index, entry }
    }
}

impl Widget for LogRow<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.style_mut().interaction.selectable_labels = false;

        let bg_color = if self.index % 2 == 1 {
            ui.visuals().faint_bg_color
        } else {
            egui::Color32::TRANSPARENT
        };

        egui::Frame::new()
            .fill(bg_color)
            .inner_margin(4.0)
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.take_available_width();

                    ui.add_sized(
                        [60.0, 0.0],
                        egui::Label::new(
                            RichText::new(
                                self.entry
                                    .timestamp
                                    .naive_local()
                                    .format("%H:%M:%S")
                                    .to_string(),
                            )
                            .color(ui.visuals().weak_text_color())
                            .family(egui::FontFamily::Monospace)
                            .size(11.0),
                        ),
                    );

                    let color = match self.entry.level {
                        LogLevel::Info => ui.visuals().text_color(),
                        LogLevel::Error => egui::Color32::from_rgb(240, 100, 100),
                        LogLevel::Success => egui::Color32::from_rgb(100, 240, 100),
                    };

                    ui.add(
                        egui::Label::new(
                            RichText::new(&self.entry.message)
                                .color(color)
                                .family(egui::FontFamily::Monospace)
                                .text_style(TextStyle::Small),
                        )
                        .wrap(),
                    );
                });
            })
            .response
    }
}
