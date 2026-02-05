use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use eframe::egui;
use eframe::egui::{Context, Memory, MenuBar, RawInput, RichText, TextStyle, warn_if_debug_build};
use eframe::egui_wgpu::RenderState;
use tokio::sync::{Notify, mpsc};

use super::capabilities::Capabilities;
use super::pollers::{spawn_adb_poller, spawn_msg_poller};
use crate::core::model::Model;
use crate::core::msg::Msg;
use crate::services::adb::AdbHandle;
use crate::ui::context::{OwnedViewContext, ViewContext};
use crate::ui::pages::home::HomePage;
use crate::ui::pages::settings::{SessionSettings, SettingsPage};
use crate::ui::pages::{CurrentPage, Page};
use crate::ui::perf::fps_counter::{FpsCounter, FrameStats};
use crate::ui::views::session::SessionViewport;
use crate::utils::math::round_magnitude;

pub type MsgSender = mpsc::UnboundedSender<Msg>;

#[derive(Serialize, Deserialize)]
pub struct AppData {
    pub label: String,
    pub value: f32,
    pub session_settings: SessionSettings,
    pub memory: Memory,
}

pub struct Octane {
    /// Logic
    core: Model,

    /// Side Effects
    capabilities: Capabilities,

    /// Message Queue
    rx: mpsc::UnboundedReceiver<Msg>,

    /// Application data
    data: AppData,

    // Internals
    counter: FpsCounter,
    counter_stats: (Instant, FrameStats),
    is_focused: Arc<AtomicBool>,
    exit_notify: Arc<Notify>,

    render_state: RenderState,
}

impl Octane {
    const EVENTS_BUFFER: usize = 100;

    pub fn new(cc: &eframe::CreationContext<'_>, exit_notify: Arc<Notify>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let (ui_tx, ui_rx) = mpsc::unbounded_channel();

        // Services

        let render_state = cc
            .wgpu_render_state
            .clone()
            .expect("WGpu must be the rendering backend");

        let capabilities = Capabilities {
            adb: AdbHandle::new(),
            tx: tx.clone(),
            ctx: cc.egui_ctx.clone(),
            state: render_state.clone(),
        };

        // Pollers

        let is_focused = Arc::new(AtomicBool::new(true));

        spawn_msg_poller(rx, ui_tx, cc.egui_ctx.clone(), Self::EVENTS_BUFFER);
        spawn_adb_poller(tx, is_focused.clone());

        // Persistence

        let data = cc
            .storage
            .and_then(|s| eframe::get_value(s, eframe::APP_KEY))
            .map_or_else(
                || AppData {
                    label: "Hello World!".to_owned(),
                    value: 2.7,
                    session_settings: SessionSettings::default(),
                    memory: Memory::default(),
                },
                |data| data,
            );
        cc.egui_ctx.memory_mut(|m| *m = data.memory.clone());

        // Styles

        // cc.egui_ctx.set_request_repaint_callback(|request| {
        //     println!("Request repaint {request:?}");
        // });

        // let font = std::fs::read("c:/Windows/Fonts/msyh.ttc").unwrap();
        // cc.egui_ctx.add_font(FontInsert::new(
        //     "Windows",
        //     FontData::from_owned(font),
        //     vec![InsertFontFamily {
        //         family: FontFamily::default(),
        //         priority: FontPriority::Highest,
        //     }],
        // ));
        // cc.egui_ctx.set_visuals(Visuals::dark());
        cc.egui_ctx.style_mut(|s| {
            s.interaction.tooltip_delay = 0.1;
            s.interaction.tooltip_grace_time = 0.0;

            s.interaction.selectable_labels = false;

            s.text_styles.insert(
                TextStyle::Body,
                egui::FontId {
                    size: 16.0,
                    family: egui::FontFamily::default(),
                },
            );
            s.text_styles.insert(
                TextStyle::Small,
                egui::FontId {
                    size: 12.0,
                    family: egui::FontFamily::default(),
                },
            );
        });

        Self {
            core: Model::new(),
            capabilities,
            data,
            rx: ui_rx,
            counter: FpsCounter::new(120, Duration::from_secs(1)),
            counter_stats: (Instant::now(), FrameStats::default()),
            is_focused,
            exit_notify,
            render_state,
        }
    }

    const fn ctx(&mut self) -> ViewContext<'_> {
        ViewContext::new(&self.core, &mut self.data, &self.capabilities.tx)
    }

    fn owned_ctx(&self) -> OwnedViewContext {
        OwnedViewContext::new(self.render_state.clone(), self.capabilities.tx.clone())
    }

    /// Tick and return the current time
    fn tick(&mut self) -> Instant {
        const INTERVAL: Duration = Duration::from_millis(100);

        let now = self.counter.tick();
        #[allow(clippy::unchecked_duration_subtraction)]
        if self.counter_stats.0 <= now - INTERVAL {
            self.counter_stats = (now, self.counter.stats());
        }
        now
    }

    /// Frame counter stats
    const fn stats(&self) -> FrameStats {
        self.counter_stats.1
    }

    fn process_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            let effects = self.core.update(msg);

            for effect in effects {
                self.capabilities.handle_effect(effect);
            }
        }
    }
}

impl eframe::App for Octane {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.is_focused
            .store(ctx.input(|i| i.focused), Ordering::Relaxed);

        self.process_messages();

        for session in self.core.sessions.values().cloned() {
            SessionViewport::new(session).show(ctx, self.owned_ctx());
        }

        let counter_stats = self.stats();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                ui.small(
                    RichText::new(format!(
                        "FPS: {:.2} P90: {:.2} MAX: {:.2}",
                        counter_stats.fps,
                        round_magnitude(counter_stats.p90.as_secs_f64(), -3, 2),
                        round_magnitude(counter_stats.max.as_secs_f64(), -3, 2)
                    ))
                    .monospace(),
                );

                // widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label(RichText::new("GitHub: ").small());
                ui.hyperlink_to(
                    RichText::new("Octane").small(),
                    "https://github.com/Octane-rs",
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    warn_if_debug_build(ui);
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.core.current_page {
            CurrentPage::Home => {
                HomePage::new().show(ui, &mut self.ctx());
            }
            CurrentPage::Settings => {
                SettingsPage::new().show(ui, &mut self.ctx());
            }
        });

        self.tick();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.data);
    }

    fn on_exit(&mut self) {
        let cap = &self.capabilities;
        let adb = cap.adb.clone();

        tokio::spawn(async move {
            adb.exit().await;
        });

        self.exit_notify.notify_one();
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        // egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).to_normalized_gamma_f32()

        visuals.window_fill().to_normalized_gamma_f32()
    }

    fn raw_input_hook(&mut self, _ctx: &Context, _raw_input: &mut RawInput) {
        // println!("Raw: {:?}", _raw_input);
    }
}
