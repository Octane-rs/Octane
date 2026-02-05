use std::fmt;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};

use crate::services::adb::{AdbHandle, DeviceId};
use crate::services::session::{SessionActor, SessionCommand, SessionConfig, SharedFrame};

/// Control configuration
#[derive(Debug, Clone)]
pub struct SessionControl;

/// Audio configuration
#[derive(Debug, Clone)]
pub struct SessionAudio;

/// Video configuration
#[derive(Debug, Clone)]
pub struct SessionVideo;

/// A thread-safe handle for interacting with the Session service.
#[derive(Clone)]
pub struct SessionHandle {
    pub device_id: DeviceId,

    /// Control events sender channel
    pub control_tx: mpsc::Sender<()>,
    /// Current video frame
    pub shared_frame: SharedFrame,

    /// Control configuration. `None` if no control stream is enabled.
    pub control: Option<SessionControl>,
    /// Audio configuration. `None` if no audio stream is enabled.
    pub audio: Option<SessionAudio>,
    /// Video configuration. `None` if no video stream is enabled.
    pub video: Option<SessionVideo>,

    sender: mpsc::Sender<SessionCommand>,
}

impl SessionHandle {
    const BUFFER: usize = 32;

    /// Spawns the Session actor and returns a communication handle.
    pub fn new(
        adb: AdbHandle,
        config: SessionConfig,
        exit_tx: oneshot::Sender<Option<anyhow::Error>>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(Self::BUFFER);
        let (control_tx, control_rx) = mpsc::channel(32);
        let shared_frame = Arc::new(parking_lot::RwLock::new(None));

        let device_id = config.device_id.clone();
        let control = config.control.as_ref().map(|()| SessionControl);
        let audio = config.audio.as_ref().map(|()| SessionAudio);
        let video = config.video.as_ref().map(|_| SessionVideo);

        let session = SessionActor::new(adb, config, control_rx, shared_frame.clone(), rx);
        tokio::spawn(async move {
            let result = session.run().await;
            let _ = exit_tx.send(result.err());
        });

        Self {
            device_id,
            control_tx,
            shared_frame,
            control,
            audio,
            video,
            sender: tx,
        }
    }

    /// Signals the Session service to exit.
    pub async fn exit(&self) {
        let _ = self.sender.send(SessionCommand::Exit).await;
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl fmt::Debug for SessionHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionHandle")
            .field("device_id", &self.device_id)
            .field("control", &self.control.is_some())
            .field("audio", &self.audio.is_some())
            .field("video", &self.video.is_some())
            .finish()
    }
}
