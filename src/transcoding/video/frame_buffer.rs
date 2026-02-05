use std::ops::{Deref, DerefMut};

use ffmpeg_next::{encoder, ffi, frame};

use crate::transcoding::error::FFmpegError;
use crate::transcoding::ffmpeg_result;

pub enum FrameBuffer {
    /// Hardware video frame.
    Hw(frame::Video),
    /// Software video frame.
    Sw(frame::Video),
}

impl FrameBuffer {
    /// Video frame.
    pub fn frame(&self) -> &frame::Video {
        self
    }

    /// Video frame.
    pub fn frame_mut(&mut self) -> &mut frame::Video {
        self
    }

    /// Download a frame to the CPU.
    /// No change will occur if the frame is already on the CPU.
    pub fn download_to_cpu(&mut self) -> Result<(), FFmpegError> {
        if let Self::Hw(hw_frame) = self {
            let mut sw_frame = frame::Video::empty();

            let (hw_frame_ptr, sw_frame_ptr) =
                unsafe { (hw_frame.as_mut_ptr(), sw_frame.as_mut_ptr()) };

            // let hw_formats = Some(unsafe { hw::get_transfer_formats_from_gpu(hw_frame_ptr) });
            // info!("Hardware transfer formats from GPU: {hw_formats:?}");

            // Transfer data to CPU
            ffmpeg_result!(ffi::av_hwframe_transfer_data(sw_frame_ptr, hw_frame_ptr, 0); FromHWTransferError);

            copy_frame_props(sw_frame_ptr, hw_frame_ptr)?;

            *self = Self::Sw(sw_frame);
        }

        Ok(())
    }

    /// Download a frame to the GPU.
    /// No change will occur if the frame is already on the GPU.
    pub fn download_to_gpu(
        &mut self,
        encoder: &mut encoder::video::Video,
    ) -> Result<(), FFmpegError> {
        if let Self::Sw(sw_frame) = self {
            let mut hw_frame = frame::Video::empty();

            let (hw_frame_ptr, sw_frame_ptr) = unsafe {
                if (*encoder.as_mut_ptr()).hw_frames_ctx.is_null() {
                    return Err(FFmpegError::HWFramesContextMissing);
                }

                (hw_frame.as_mut_ptr(), sw_frame.as_mut_ptr())
            };

            // Allocate hardware buffer
            ffmpeg_result!(ffi::av_hwframe_get_buffer(
                    (*encoder.as_mut_ptr()).hw_frames_ctx,
                    hw_frame_ptr,
                    0,
                ); ToHWTransferError);
            // Transfer data to GPU
            ffmpeg_result!(ffi::av_hwframe_transfer_data(
                    hw_frame_ptr,
                    sw_frame_ptr,
                    0
                ); ToHWTransferError);

            copy_frame_props(hw_frame_ptr, sw_frame_ptr)?;

            *self = Self::Hw(hw_frame);
        }

        Ok(())
    }
}

impl Deref for FrameBuffer {
    type Target = frame::Video;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Hw(f) | Self::Sw(f) => f,
        }
    }
}

impl DerefMut for FrameBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Hw(f) | Self::Sw(f) => f,
        }
    }
}

impl From<FrameBuffer> for frame::Video {
    fn from(frame: FrameBuffer) -> Self {
        match frame {
            FrameBuffer::Hw(f) | FrameBuffer::Sw(f) => f,
        }
    }
}

fn copy_frame_props(dst: *mut ffi::AVFrame, src: *const ffi::AVFrame) -> Result<(), FFmpegError> {
    ffmpeg_result!(ffi::av_frame_copy_props(dst, src); FromHWTransferError);

    Ok(())
}
