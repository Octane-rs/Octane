use std::collections::HashMap;

use tokio::sync::{mpsc, oneshot};

use crate::services::adb::{AdbHandle, DeviceId};
use crate::services::session::{SessionConfig, SessionHandle, SessionStoppedCallback};
pub use crate::services::session_manager::error::SessionManagerError;
pub use crate::services::session_manager::handle::SessionManagerHandle;

pub mod error;
mod handle;

pub type SessionManagerResult<T = ()> = Result<T, SessionManagerError>;

enum SessionManagerCommand {
    /// Requests the start of a new session or retrieval of an existing one.
    Start {
        config: SessionConfig,
        stopped_cb: SessionStoppedCallback,
        respond_to: oneshot::Sender<SessionManagerResult<SessionHandle>>,
    },
    /// Requests the immediate termination of the device session.
    Stop { device_id: DeviceId },
    /// Signals the service to exit and close started sessions.
    Exit,
}

pub struct SessionManagerActor {
    sessions: HashMap<String, SessionHandle>,

    adb: AdbHandle,

    rx: mpsc::Receiver<SessionManagerCommand>,
}

impl SessionManagerActor {
    /// Creates a new [`SessionManagerActor`] instance.
    ///
    /// # Arguments
    ///
    /// - `adb` - Adb service handle.
    /// - `receiver` - Channel receiver for processing [`SessionManagerCommand`] messages.
    fn new(adb: AdbHandle, rx: mpsc::Receiver<SessionManagerCommand>) -> Self {
        Self {
            sessions: HashMap::new(),
            adb,
            rx,
        }
    }

    /// Runs the actor's main event loop.
    ///
    /// Processes messages sequentially until the channel is closed
    /// or a [`SessionManagerCommand::Exit`] message is received.
    async fn run(mut self) {
        while let Some(cmd) = self.rx.recv().await {
            match cmd {
                SessionManagerCommand::Start {
                    config,
                    stopped_cb,
                    respond_to,
                } => {
                    let device_id = config.device_id.clone();
                    let session = if let Some(s) = self.sessions.get(&device_id) {
                        debug!("Existing session \"{device_id}\".");
                        s.clone()
                    } else {
                        debug!("Starting session \"{device_id}\".");

                        let session = SessionHandle::new(self.adb.clone(), config, stopped_cb);
                        self.sessions.insert(device_id, session.clone());

                        session
                    };
                    let _ = respond_to.send(Ok(session));
                }
                SessionManagerCommand::Stop { device_id } => {
                    debug!("Stopping session \"{device_id}\".");
                    self.sessions.remove(&device_id);
                }
                SessionManagerCommand::Exit => {
                    debug!("Shutting down, {} messages remaining.", self.rx.len());
                    self.rx.close();
                    break;
                }
            }
        }

        self.exit().await;
    }

    /// Performs a clean exit sequence.
    async fn exit(mut self) {
        debug!("Initiating shutdown.");

        if self.sessions.is_empty() {
            debug!("No session stop.");
        } else {
            for (_, session) in self.sessions.drain() {
                debug!("Stopping session \"{}\".", session.device_id);
                session.exit().await;
            }
        }
    }
}
