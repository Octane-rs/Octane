use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct FpsCounter {
    frames: VecDeque<Instant>,
    sorted_frames: Vec<Duration>,
    capacity: usize,
    window_duration: Duration,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FrameStats {
    pub avg: Duration,
    pub min: Duration,
    pub max: Duration,
    pub p90: Duration,
    pub p99: Duration,
    pub fps: f64,
}

impl FpsCounter {
    /// In milliseconds
    ///
    /// # Panics
    ///
    /// Panics when capacity = 0.
    pub fn new(capacity: usize, window_duration: Duration) -> Self {
        assert!(capacity > 0);

        Self {
            frames: VecDeque::with_capacity(capacity),
            sorted_frames: Vec::with_capacity(capacity),
            capacity,
            window_duration,
        }
    }

    /// Tick and return the current time
    pub fn tick(&mut self) -> Instant {
        let now = Instant::now();

        if self.frames.len() == self.capacity {
            self.frames.pop_front();
        }

        self.frames.push_back(now);

        let limit = now.checked_sub(self.window_duration).unwrap_or(now);

        while let Some(&front) = self.frames.front() {
            if front < limit {
                self.frames.pop_front();
            } else {
                break;
            }
        }

        now
    }

    pub fn stats(&mut self) -> FrameStats {
        if self.frames.len() < 2 {
            return FrameStats::default();
        }

        self.sorted_frames.clear();
        for (&c, &p) in self.frames.iter().skip(1).zip(self.frames.iter()) {
            self.sorted_frames.push(c - p);
        }
        self.sorted_frames.sort_unstable();

        let len = self.sorted_frames.len();

        let oldest = *self.frames.back().expect("Frames len >= 2");
        let newest = *self.frames.front().expect("Frames len >= 2");
        let duration = oldest - newest;

        let avg = duration.div_f64(len as f64);
        let min = self.sorted_frames[0];
        let max = self.sorted_frames[len - 1];

        let p90_i = ((len as f64 * 0.90) as usize).min(len - 1);
        let p99_i = ((len as f64 * 0.99) as usize).min(len - 1);
        let p90 = self.sorted_frames[p90_i];
        let p99 = self.sorted_frames[p99_i];

        FrameStats {
            avg,
            min,
            max,
            p90,
            p99,
            fps: 1.0 / avg.as_secs_f64().max(0.00001),
        }
    }
}
