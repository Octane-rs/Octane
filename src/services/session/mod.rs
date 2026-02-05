use std::sync::Arc;
use std::{fmt, io};

use ffmpeg_next::codec;
use scrcpy_launcher::ScrcpyLauncher;
use scrcpy_launcher::options::{Options, ServerId, VideoCodec};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::services::adb::{AdbHandle, DeviceId};
pub use crate::services::session::handle::SessionHandle;
use crate::services::stream_decoder::VideoStreamDecoder;
use crate::transcoding::video::frame_buffer::FrameBuffer;

mod handle;

pub type SharedFrame = Arc<parking_lot::RwLock<Option<FrameBuffer>>>;
pub type FrameCallback = Box<dyn Fn() + Send + Sync>;
pub type SessionStoppedCallback = Box<dyn FnOnce(Option<anyhow::Error>) + Send + Sync>;

#[derive(Debug, PartialEq, Eq)]
pub enum SessionCommand {
    /// Signals the service to exit the current session.
    Exit,
}

pub struct SessionActor {
    config: SessionConfig,

    adb: AdbHandle,

    control_rx: mpsc::Receiver<()>,
    video_tx: Option<OwnedWriteHalf>,
    shared_frame: SharedFrame,

    rx: mpsc::Receiver<SessionCommand>,
    set: JoinSet<io::Result<()>>,
}

#[derive(Debug)]
pub struct SessionConfig {
    pub device_id: DeviceId,
    pub control: Option<()>,
    pub audio: Option<()>,
    pub video: Option<SessionVideoConfig>,
}

pub struct SessionVideoConfig {
    pub codec: VideoCodec,
    pub width: i32,
    pub bitrate: i32,
    pub max_fps: f32,
    pub hw_decoder: bool,
    pub on_frame_cb: FrameCallback,
}

impl SessionActor {
    const PORT: u16 = 3333;

    pub fn new(
        adb: AdbHandle,
        config: SessionConfig,
        control_rx: mpsc::Receiver<()>,
        shared_frame: SharedFrame,
        rx: mpsc::Receiver<SessionCommand>,
    ) -> Self {
        Self {
            config,
            adb,
            control_rx,
            video_tx: None,
            shared_frame,
            rx,
            set: JoinSet::new(),
        }
    }

    /// Runs the actor's main event loop.
    ///
    /// Processes the session audio, video and control streams, until
    /// the channel is closed or a [`SessionCommand::Exit`] message is received.
    async fn run(mut self) -> Result<(), anyhow::Error> {
        let device = self.adb.get_device(self.config.device_id.clone()).await?;

        let mut options = Options::new()
            .set_control(false)
            .set_audio(false)
            .set_video(false)
            .set_scid(ServerId::random());

        if let Some(_config) = self.config.control.as_ref() {
            options = options.set_control(true);
        }
        if let Some(_config) = self.config.audio.as_ref() {
            options = options.set_audio(true);
        }
        if let Some(config) = self.config.video.as_ref() {
            options = options
                .set_video(true)
                // .set_new_display(Some(NewDisplay::new_size(1080, 1080)))
                // .set_vd_system_decorations(false)
                .set_video_codec(config.codec)
                .set_max_size(config.width)
                .set_video_bit_rate(config.bitrate)
                .set_max_fps(config.max_fps);
        }

        debug!("{options:#?}");

        let launcher = ScrcpyLauncher::new(device, Self::PORT);
        let (connection, _process) = launcher.start(options).await?;
        let mut session = connection.start().await?;

        match (self.config.video, session.get_video_mut().take()) {
            (None, None) => {}
            (Some(config), Some(video)) => {
                // NOTE: the writer must not be dropped.
                self.video_tx.replace(video.state.tx);
                let video_rx = video.state.rx;
                let metadata = video.state.metadata;

                let codec = match config.codec {
                    VideoCodec::H264 => codec::Id::H264,
                    VideoCodec::H265 => codec::Id::H265,
                    VideoCodec::AV1 => codec::Id::AV1,
                };
                let size = (metadata.width, metadata.height);
                let decoder = VideoStreamDecoder::new(codec, size, config.hw_decoder);
                decoder.start(
                    &mut self.set,
                    video_rx,
                    Box::new(move |frame| {
                        self.shared_frame.write().replace(frame);
                        (config.on_frame_cb)();
                    }),
                );
            }
            _ => unreachable!("Video configuration mismatch"),
        }

        tokio::select! {
            // Internal join set has ended. Likely a failure in the consumer pipelines.
            results = self.set.join_all() => {
                error!("Session unexpectedly ended for \"{}\" {results:?}", self.config.device_id);
                for result in results {
                    if let Err(err) = result {
                        return Err(err.into());
                    }
                }
            }
            // Session has ended, likely a user device disconnect.
            results = session.join() => {
                info!("Session ended for \"{}\" {results:?}", self.config.device_id);
                for result in results {
                    if let Err(err) = result {
                        return Err(err.into());
                    }
                }
            }
            // User stop signal from the ui.
            command = self.rx.recv() => if command == Some(SessionCommand::Exit) {
                info!("Stopping session for \"{}\"", self.config.device_id);
            }
        }

        Ok(())
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl fmt::Debug for SessionVideoConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionVideoConfig")
            .field("codec", &self.codec)
            .field("width", &self.width)
            .field("bitrate", &self.bitrate)
            .field("max_fps", &self.max_fps)
            .finish()
    }
}
