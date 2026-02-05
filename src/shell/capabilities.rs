use eframe::egui::Context;
use eframe::egui_wgpu::RenderState;
use tokio::sync::oneshot;

use crate::core::effect::{Effect, LogLevel};
use crate::core::msg::Msg;
use crate::core::primitives::async_state::AsyncResult;
use crate::services::adb::AdbHandle;
use crate::services::session::SessionHandle;
use crate::shell::app::MsgSender;

pub struct Capabilities {
    pub(super) adb: AdbHandle,

    pub(super) state: RenderState,
    pub(super) ctx: Context,
    pub(super) tx: MsgSender,
}

impl Capabilities {
    pub fn handle_effect(&self, effect: Effect) {
        match effect {
            Effect::Render => {
                self.ctx.request_repaint();
            }

            Effect::Log { level, msg } => match level {
                LogLevel::Trace => trace!("{msg}"),
                LogLevel::Info => info!("{msg}"),
                LogLevel::Error => error!("{msg}"),
            },

            Effect::FetchAdbDevices { ticket } => {
                let adb = self.adb.clone();
                let tx = self.tx.clone();

                tokio::spawn(async move {
                    let result = adb.get_devices().await;
                    let _ = tx.send(Msg::AdbDevicesLoaded(AsyncResult { ticket, result }));
                });
            }

            Effect::StartSession { config } => {
                let adb = self.adb.clone();
                let tx = self.tx.clone();
                let device_id = config.device_id.clone();

                tokio::spawn(async move {
                    let (exit_tx, exit_rx) = oneshot::channel();

                    let session = SessionHandle::new(adb, config, exit_tx);
                    let _ = tx.send(Msg::SessionStarted { session });

                    let error = exit_rx.await.unwrap_or_default();
                    let _ = tx.send(Msg::SessionStopped { device_id, error });
                });
            }
            Effect::StopSession { session } => {
                tokio::spawn(async move {
                    session.exit().await;
                });
            }
        }
    }

    pub fn send(&self, msg: Msg) {
        let _ = self.tx.send(msg);
    }
}
