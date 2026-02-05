#![allow(unsafe_code)]

use ffmpeg_next::{Dictionary, Error, Packet, codec, decoder, ffi, frame, threading};

use crate::transcoding::error::FFmpegError;
use crate::transcoding::hw::device::HWDevice;
use crate::transcoding::video::frame_buffer::FrameBuffer;

pub struct VideoDecoder<'a> {
    /// Video decoder context
    pub decoder: decoder::Video,
    /// Optional hardware device for hardware-accelerated decoding
    pub hw_device: Option<(&'a HWDevice, (i32, i32))>,
}

impl<'a> VideoDecoder<'a> {
    /// Create a new video decoder from the specified codec ID and options.
    ///
    /// # Arguments
    /// - `codec`: The codec ID to use for decoding.
    /// - `options`: A dictionary of options to configure the decoder.
    /// - `hw_device`: An optional hardware device to use for hardware-accelerated decoding.
    ///   `(HWDevice, (width, height))` for hardware decoding,
    ///   `None` for software decoding.
    pub fn from_codec(
        codec: codec::Id,
        options: &Dictionary<'_>,
        hw_device: Option<(&'a HWDevice, (i32, i32))>,
    ) -> Result<Self, FFmpegError> {
        // Find the decoder for the specified codec ID
        let decoder = unsafe {
            let decoder = ffi::avcodec_find_decoder(codec.into());
            if decoder.is_null() {
                return Err(FFmpegError::DecoderNotFound);
            }
            decoder
        };

        // Create decoder context
        let mut decoder_ctx =
            unsafe { codec::context::Context::wrap(ffi::avcodec_alloc_context3(decoder), None) };
        // `threading::Type::Frame` increase delay by one frame, do not use
        decoder_ctx.set_threading(threading::Config {
            kind: threading::Type::Slice,
            count: 5,
        });
        decoder_ctx.set_flags(codec::Flags::LOW_DELAY);

        // Apply decoder options
        for (k, v) in options.iter() {
            unsafe {
                let k_cstr = std::ffi::CString::new(k).unwrap_or_default();
                let v_cstr = std::ffi::CString::new(v).unwrap_or_default();
                ffi::av_opt_set(
                    (*decoder_ctx.as_mut_ptr()).priv_data,
                    k_cstr.as_ptr(),
                    v_cstr.as_ptr(),
                    0,
                );
            }
        }

        if let Some((hw_device, size)) = hw_device {
            hw_device.bind_transcoder(&mut decoder_ctx, size)?;
        }

        Ok(Self {
            decoder: decoder_ctx.decoder().open()?.video()?,
            hw_device,
        })
    }

    pub fn send_packet(&mut self, packet: &Packet) -> Result<(), FFmpegError> {
        self.decoder.send_packet(packet).map_err(Into::into)
    }

    pub fn receive_frame(&mut self) -> Result<FrameBuffer, FFmpegError> {
        let mut frame = frame::Video::empty();
        if let Err(err) = self.decoder.receive_frame(&mut frame) {
            return Err(err.into());
        }

        let frame_buffer = if self.hw_device.is_some()
            && unsafe { !(*frame.as_mut_ptr()).hw_frames_ctx.is_null() }
        {
            FrameBuffer::Hw(frame)
        } else {
            FrameBuffer::Sw(frame)
        };

        // Handle color range for JPEG formats
        // let frame = &mut *frame_buffer;
        //
        // if frame.format() == format::Pixel::YUVJ420P {
        //     frame.set_format(format::Pixel::YUV420P);
        //     frame.set_color_range(util::color::Range::JPEG);
        // }

        Ok(frame_buffer)
    }

    pub fn receive_frames<F>(&mut self, mut process_frame: F) -> Result<(), FFmpegError>
    where
        F: FnMut(FrameBuffer) -> Result<(), FFmpegError>,
    {
        loop {
            match self.receive_frame() {
                Ok(frame_buffer) => process_frame(frame_buffer)?,
                Err(FFmpegError::InternalError(Error::Eof | Error::Exit | Error::Other { .. })) => {
                    break;
                }
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }

    /// Send null packet to flush decoder
    pub fn send_eof(&mut self) -> Result<(), FFmpegError> {
        let mut packet = Packet::empty();
        packet.set_pts(None);

        self.send_packet(&packet)
    }

    pub fn flush<F>(&mut self, process_frame: F) -> Result<(), FFmpegError>
    where
        F: FnMut(FrameBuffer) -> Result<(), FFmpegError>,
    {
        self.send_eof()?;

        // Process remaining frames
        self.receive_frames(process_frame)
    }
}
