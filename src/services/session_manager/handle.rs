use tokio::sync::{mpsc, oneshot};

use crate::services::adb::{AdbHandle, DeviceId};
use crate::services::helpers::sender::Sender;
use crate::services::session::{SessionConfig, SessionHandle, SessionStoppedCallback};
use crate::services::session_manager::{
    SessionManagerActor, SessionManagerCommand, SessionManagerError, SessionManagerResult,
};

/// A thread-safe handle for interacting with the Session manager service.
#[derive(Clone)]
pub struct SessionManagerHandle {
    tx: Sender<SessionManagerCommand>,
}

impl SessionManagerHandle {
    const BUFFER: usize = 32;

    /// Spawns the Session manager actor and returns a communication handle.
    pub fn new(adb: AdbHandle) -> Self {
        Self {
            tx: Sender::new("SessionManagerActor", move || {
                let (tx, rx) = mpsc::channel(Self::BUFFER);
                tokio::spawn(SessionManagerActor::new(adb.clone(), rx).run());
                tx
            }),
        }
    }

    /// Requests the start of a new session or retrieval of an existing one.
    pub async fn start(
        &self,
        config: SessionConfig,
        stopped_cb: SessionStoppedCallback,
    ) -> SessionManagerResult<SessionHandle> {
        let (tx, rx) = oneshot::channel();
        let cmd = SessionManagerCommand::Start {
            config,
            stopped_cb,
            respond_to: tx,
        };

        self.tx
            .send(cmd)
            .await
            .map_err(|_| SessionManagerError::ChannelClosed)?;

        rx.await.map_err(|_| SessionManagerError::ChannelClosed)?
    }

    /// Requests the immediate termination of the device session.
    pub async fn stop(&self, device_id: DeviceId) -> SessionManagerResult {
        self.tx
            .send(SessionManagerCommand::Stop { device_id })
            .await
            .map_err(|_| SessionManagerError::ChannelClosed)
    }

    /// Signals the service to exit itself and the started sessions.
    pub async fn exit(&self) {
        let _ = self.tx.send(SessionManagerCommand::Exit).await;
    }
}
