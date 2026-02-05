use std::io;

use ffmpeg_next::{Dictionary, Packet, codec, ffi};
use scrcpy_launcher::video::{FrameMetadata, PacketType};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::transcoding::hw::device_pool::HWDevicePool;
use crate::transcoding::video::decoder::VideoDecoder;
use crate::transcoding::video::frame_buffer::FrameBuffer;

pub type FrameCallback = Box<dyn Fn(FrameBuffer) + Send + Sync + 'static>;

/// NAL video stream decoder
pub struct VideoStreamDecoder {
    /// Decoding codec ID
    codec: codec::Id,
    /// Video frames size in pixels (width, height)
    size: (i32, i32),
    /// Whether to use Vulkan hardware decoder
    hw_decoder: bool,
}

impl VideoStreamDecoder {
    /// Creates a new NAL video stream decoder
    ///
    /// # Arguments
    ///
    /// - `codec`: Decoding codec ID
    /// - `size`: Video frames size in pixels (width, height)
    pub const fn new(codec: codec::Id, size: (i32, i32), hw_decoder: bool) -> Self {
        Self {
            codec,
            size,
            hw_decoder,
        }
    }

    /// Spawns background decoding task.
    ///
    /// Consumes a NAL stream, overwriting mailbox with latest frame and triggering update signal.
    ///
    /// # Arguments
    ///
    /// - `set`: Task lifecycle manager.
    /// - `stream`: Incoming NAL unit source.
    /// - `on_frame`: Decoded frame callback.
    pub fn start(
        self,
        set: &mut JoinSet<io::Result<()>>,
        mut stream: mpsc::Receiver<(FrameMetadata, Vec<u8>)>,
        on_frame: FrameCallback,
    ) {
        set.spawn_blocking(move || {
            // Vulkan is expected to be supported since it is used by the ui.
            let Some(hw_device) = HWDevicePool::first(ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_VULKAN)
            else {
                error!("No Vulkan hardware device available");
                return Ok(());
            };

            let result = VideoDecoder::from_codec(
                self.codec,
                &Dictionary::default(),
                self.hw_decoder.then_some((hw_device, self.size)),
            );
            let mut decoder = match result {
                Ok(decoder) => decoder,
                Err(err) => {
                    error!("Failed to start the video decoder: {err}");
                    return Ok(());
                }
            };

            while let Some((metadata, buffer)) = stream.blocking_recv() {
                let packet = new_packet(&metadata, &buffer);

                if let Err(err) = decoder.send_packet(&packet) {
                    error!("Send packet: {err}");
                    continue;
                }

                // Config packet are not frames
                if metadata.packet == PacketType::Config {
                    continue;
                }

                match decoder.receive_frame() {
                    Ok(frame) => on_frame(frame),
                    Err(err) => error!("Receive frame: {err}"),
                }
            }

            Ok(())
        });
    }
}

/// Creates an owned `FFmpeg` video packet
pub fn new_packet(metadata: &FrameMetadata, data: &[u8]) -> Packet {
    assert_eq!(metadata.size as usize, data.len(), "Invalid data");

    // let mut packet = Packet::new(metadata.size as usize);
    // packet.data_mut().unwrap().write_all(data).unwrap();

    let mut packet = Packet::copy(data);

    let pts = match metadata.packet {
        PacketType::Config => None,
        PacketType::KeyFrame(pts) => {
            packet.set_flags(ffmpeg_next::packet::flag::Flags::KEY);
            Some(pts)
        }
        PacketType::Regular(pts) => Some(pts),
    };

    packet.set_pts(pts);
    packet.set_dts(pts);

    packet
}
