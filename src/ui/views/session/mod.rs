use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui;
use eframe::egui::{CentralPanel, Context, Id, RichText, TextWrapMode, ViewportId};

use crate::core::msg::Msg;
use crate::ui::context::OwnedViewContext;
use crate::ui::perf::fps_counter::FrameStats;
use crate::ui::views::session::state::SessionState;
use crate::utils::math::round_magnitude;

pub mod state;

pub struct SessionViewport {
    state: Arc<parking_lot::RwLock<SessionState>>,
}

impl SessionViewport {
    pub const fn new(state: Arc<parking_lot::RwLock<SessionState>>) -> Self {
        Self { state }
    }

    fn viewport_id(&self) -> ViewportId {
        ViewportId::from_hash_of(self.state.read().session.device_id.as_str())
    }

    /// Tick and return the current time
    fn tick(&self) -> Instant {
        const INTERVAL: Duration = Duration::from_millis(100);

        let mut state = self.state.write();

        let now = state.counter.tick();
        #[allow(clippy::unchecked_duration_subtraction)]
        if state.counter_stats.0 <= now - INTERVAL {
            state.counter_stats = (now, state.counter.stats());
        }
        now
    }

    /// Frame counter stats
    fn stats(&self) -> FrameStats {
        self.state.read().counter_stats.1
    }

    pub fn show(self, context: &Context, ctx: OwnedViewContext) {
        context.show_viewport_deferred(
            self.viewport_id(),
            egui::ViewportBuilder::default()
                .with_title(format!(
                    "Device \"{}\"",
                    self.state.read().session.device_id
                ))
                .with_inner_size([500.0, 500.0])
                .with_min_inner_size([200.0, 100.0]),
            move |context, class| {
                if class == egui::ViewportClass::Embedded {
                    CentralPanel::default().show(context, |ui| {
                        ui.label("Embedded viewport are not supported.");
                    });
                    return;
                }

                let _now = self.tick();
                let counter_stats = self.stats();

                let state = &mut *self.state.write();

                egui::Area::new(Id::new("fps_overlay"))
                    .fixed_pos(egui::pos2(5.0, 5.0))
                    .order(egui::Order::Foreground)
                    .show(context, |ui| {
                        ui.available_width();

                        ui.add(
                            egui::Label::new(
                                RichText::new(format!(
                                    "FPS: {:.2} P90: {:.2} MAX: {:.2}",
                                    counter_stats.fps,
                                    round_magnitude(counter_stats.p90.as_secs_f64(), -3, 2),
                                    round_magnitude(counter_stats.max.as_secs_f64(), -3, 2)
                                ))
                                .monospace()
                                .small(),
                            )
                            .wrap_mode(TextWrapMode::Extend),
                        );
                    });

                let frame = state.session.shared_frame.write().take();
                if let (Some(player), Some(mut frame)) =
                    (state.player_mut(ctx.state.clone()), frame)
                {
                    match frame.download_to_cpu() {
                        Ok(()) => player.update(&frame),
                        Err(err) => {
                            error!("Ffmpeg: {err:?}");
                        }
                    }
                }

                CentralPanel::default()
                    .frame(egui::Frame::new().fill(egui::Color32::BLACK))
                    .show(context, |ui| {
                        if let Some(player) = &state.player_mut(ctx.state.clone()) {
                            player.ui(ui);
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.spinner();
                            });
                        }

                        if ui.input(|i| i.viewport().close_requested()) {
                            ctx.send(Msg::RequestStopSession(state.session.device_id.clone()));
                        }
                    });
            },
        );
    }
}
