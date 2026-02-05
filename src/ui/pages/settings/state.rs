use scrcpy_launcher::options;

use crate::services::session::{FrameCallback, SessionConfig, SessionVideoConfig};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SessionSettings {
    pub control_enabled: bool,
    pub audio_enabled: bool,
    pub video_enabled: bool,
    pub codec: VideoCodec,
    pub bitrate_mbps: u32,
    pub max_fps: u32,
    pub limit_resolution: u32,
    pub hw_decoder: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    H264,
    H265,
    AV1,
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            control_enabled: false,
            audio_enabled: false,
            video_enabled: true,
            codec: VideoCodec::H264,
            bitrate_mbps: 8,
            max_fps: 0,
            limit_resolution: 0,
            hw_decoder: false,
        }
    }
}

impl SessionSettings {
    #[allow(clippy::cast_precision_loss)]
    pub fn to_config(&self, device_id: String, on_frame_cb: FrameCallback) -> SessionConfig {
        if self.control_enabled {
            Some(())
        } else {
            None
        };

        if self.audio_enabled {
            Some(())
        } else {
            None
        };

        let video = if self.video_enabled {
            Some(SessionVideoConfig {
                codec: match self.codec {
                    VideoCodec::H264 => options::VideoCodec::H264,
                    VideoCodec::H265 => options::VideoCodec::H265,
                    VideoCodec::AV1 => options::VideoCodec::AV1,
                },
                width: self.limit_resolution.cast_signed(),
                bitrate: (self.bitrate_mbps * 1_000_000).cast_signed(),
                max_fps: self.max_fps as f32,
                hw_decoder: self.hw_decoder,
                on_frame_cb,
            })
        } else {
            None
        };

        SessionConfig {
            device_id,
            control: None,
            audio: None,
            video,
        }
    }
}
