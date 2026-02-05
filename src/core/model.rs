use std::collections::HashMap;
use std::sync::Arc;

use scrcpy_launcher::adb::server::DeviceLong;

use super::effect::Effect;
use super::msg::Msg;
use crate::core::logs::LogStore;
use crate::core::primitives::async_state::AsyncState;
use crate::services::adb::AdbError;
use crate::ui::pages::CurrentPage;
use crate::ui::views::session::state::{SessionState, SharedSessionState};
use crate::utils::plural;

pub struct Model {
    pub current_page: CurrentPage,

    pub adb_devices: AsyncState<Vec<DeviceLong>, AdbError>,

    // We store the generic "SharedSessionState" which the View uses.
    // The "Handle" part of the session is implicitly inside this state or managed by Shell.
    pub sessions: HashMap<String, SharedSessionState>,

    pub logs: LogStore,
}

impl Model {
    pub fn new() -> Self {
        Self {
            current_page: CurrentPage::Home,
            adb_devices: AsyncState::new(),
            sessions: HashMap::new(),
            logs: LogStore::new(1000),
        }
    }

    pub fn update(&mut self, msg: Msg) -> Vec<Effect> {
        let mut effects = vec![];

        match msg {
            // Navigation
            Msg::Navigate(page) => {
                self.logs.info(format!("Navigating to page \"{page}\""));
                self.current_page = page;
                effects.push(Effect::Render);
            }

            // ADB
            Msg::RequestAdbDevices => {
                let ticket = self.adb_devices.start_load();
                effects.push(Effect::FetchAdbDevices { ticket });
            }
            Msg::AdbDevicesLoaded(result) => {
                let trace = self
                    .adb_devices
                    .apply_trace(result, "ADB devices loaded", |v| {
                        let count = v.len();
                        let devices = plural(count, "device", "devices");
                        format!("{count} {devices}").into()
                    });
                self.logs.trace(trace);
                effects.push(Effect::Render);
            }

            // Session Start
            Msg::RequestStartSession(config) => {
                effects.push(Effect::StartSession { config });
            }
            Msg::SessionStarted { session } => {
                let device_id = session.device_id.clone();

                let mut msg = format!("Session \"{device_id}\" started: ");
                if session.control.is_some() {
                    msg.push_str(" control");
                }
                if session.audio.is_some() {
                    msg.push_str(" audio");
                }
                if session.video.is_some() {
                    msg.push_str(" video");
                }

                let session_state = Arc::new(parking_lot::RwLock::new(SessionState::new(session)));
                self.sessions.insert(device_id, session_state);

                self.logs.success(msg);
                effects.push(Effect::Render);
            }

            // Session Stop
            Msg::RequestStopSession(device_id) => {
                if let Some(state) = self.sessions.get(&device_id).cloned() {
                    let session = state.write().session.clone();
                    effects.push(Effect::StopSession { session });
                }
                effects.push(Effect::Render);
            }
            Msg::SessionStopped { device_id, error } => {
                self.sessions.remove(&device_id);

                if let Some(err) = error {
                    self.logs
                        .error(format!("Session \"{device_id}\" stopped with error: {err}"));
                } else {
                    self.logs.info(format!("Session \"{device_id}\" ended."));
                }
                effects.push(Effect::Render);
            }

            // Misc
            Msg::ClearLogs => {
                self.logs.clear();
                effects.push(Effect::Render);
            }
        }

        effects
    }
}
