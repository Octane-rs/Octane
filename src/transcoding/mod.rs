//! Low-level media processing wrapper

pub mod error;
pub mod hw;
// pub mod libavutil;
pub mod video;

macro_rules! ffmpeg_result {
    ($func:stmt; $err:ident) => {
        let err = unsafe { $func };
        if err < 0 {
            return Err(FFmpegError::$err(err));
        }
    };
}
use ffmpeg_result;
