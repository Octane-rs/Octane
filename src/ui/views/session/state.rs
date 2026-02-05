use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui_wgpu::RenderState;

use crate::services::session::SessionHandle;
use crate::ui::perf::fps_counter::{FpsCounter, FrameStats};
use crate::ui::renderer::video_player::VideoPlayer;

pub type SharedSessionState = Arc<parking_lot::RwLock<SessionState>>;

/// Session viewport state
pub struct SessionState {
    pub session: SessionHandle,
    player: Option<VideoPlayer>,
    pub counter: FpsCounter,
    pub counter_stats: (Instant, FrameStats),
}

impl SessionState {
    pub fn new(session: SessionHandle) -> Self {
        Self {
            session,
            player: None,
            counter: FpsCounter::new(120, Duration::from_secs(1)),
            counter_stats: (Instant::now(), FrameStats::default()),
        }
    }

    pub fn player_mut(&mut self, state: RenderState) -> Option<&mut VideoPlayer> {
        match (self.session.video.is_some(), self.player.is_some()) {
            (true, false) => Some(self.player.insert(VideoPlayer::new(state))),
            (_, _) => self.player.as_mut(),
        }
    }
}
