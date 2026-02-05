use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use eframe::egui;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::core::msg::Msg;
use crate::shell::app::MsgSender;

/// Wake and repaint the UI on each incoming event
pub fn spawn_msg_poller(
    mut rx: mpsc::UnboundedReceiver<Msg>,
    core_tx: mpsc::UnboundedSender<Msg>,
    ctx: egui::Context,
    buffer_size: usize,
) {
    tokio::spawn(async move {
        let mut messages = Vec::with_capacity(buffer_size);

        loop {
            if rx.recv_many(&mut messages, buffer_size).await == 0 {
                break;
            }

            #[allow(clippy::iter_with_drain)]
            for msg in messages.drain(..) {
                debug!("Msg: {msg}");
                let _ = core_tx.send(msg);
            }
            ctx.request_repaint();
        }
    });
}

/// Polls ADB periodically if window is focused
pub fn spawn_adb_poller(tx: MsgSender, is_focused: Arc<AtomicBool>) {
    const INTERVAL: Duration = Duration::from_secs(2);

    tokio::spawn(async move {
        loop {
            if is_focused.load(Ordering::Relaxed) {
                if tx.send(Msg::RequestAdbDevices).is_err() {
                    break;
                }
                sleep(INTERVAL).await;
            } else {
                sleep(Duration::from_millis(100)).await;
            }
        }
    });
}
