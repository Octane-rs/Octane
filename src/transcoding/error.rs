use std::fmt;

#[derive(Debug)]
pub enum FFmpegError {
    ConverterEmpty,
    // EncoderNotFound,
    DecoderNotFound,
    FormatNotSupported,
    // NoSupportedFormats,
    // NoOutputContext,
    // EncoderConverterEmpty,
    FrameEmpty,
    // NoGPUDecodingDevice,
    // NoHWTransferFormats,
    FromHWTransferError(i32),
    ToHWTransferError(i32),
    HWDeviceCreateError(i32),
    HWDeviceConstraintsUnavailable,
    HWFramesContextCreationError,
    HWFramesContextMissing,
    // GPUDecodingFailed,
    // ToHWBufferError(i32),
    // PixelFormatNotSupported((format::Pixel, Vec<format::Pixel>, Option<format::Pixel>)),
    // UnknownPixelFormat(format::Pixel),
    InternalError(ffmpeg_next::Error),
}

impl From<ffmpeg_next::Error> for FFmpegError {
    fn from(err: ffmpeg_next::Error) -> Self {
        Self::InternalError(err)
    }
}

impl fmt::Display for FFmpegError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ConverterEmpty => "Converter is null",
            // Self::EncoderNotFound => "Encoder not found",
            Self::DecoderNotFound => "Decoder not found",
            Self::FormatNotSupported => "Format not supported",
            // Self::NoSupportedFormats => "No supported formats",
            // Self::NoOutputContext => "No output context",
            // Self::EncoderConverterEmpty => "Encoder converter is null",
            Self::FrameEmpty => "Frame is null",
            // Self::NoGPUDecodingDevice => "Unable to create any HW decoding context",
            // Self::NoHWTransferFormats => "No hardware transfer formats",
            Self::FromHWTransferError(i) => {
                return write!(
                    f,
                    "Error transferring frame from the GPU: {}",
                    ffmpeg_next::Error::Other { errno: *i }
                );
            }
            Self::ToHWTransferError(i) => {
                return write!(
                    f,
                    "Error transferring frame to the GPU: {}",
                    ffmpeg_next::Error::Other { errno: *i }
                );
            }
            // Self::ToHWBufferError(i) => {
            //     return write!(
            //         f,
            //         "Error getting HW transfer buffer to the GPU: {}",
            //         ffmpeg_next::Error::Other { errno: *i }
            //     );
            // }
            // Self::GPUDecodingFailed => "GPU decoding failed, please try again.",
            Self::HWDeviceCreateError(i) => {
                return write!(
                    f,
                    "Unable to create HW devices context: {}",
                    ffmpeg_next::Error::Other { errno: *i }
                );
            }
            Self::HWDeviceConstraintsUnavailable => "HW device constraints are unavailable",
            Self::HWFramesContextCreationError => "HW frames context creation error",
            Self::HWFramesContextMissing => "HW frames context missing",
            // Self::UnknownPixelFormat(v) => return write!(f, "Unknown pixel format: {v}"),
            // Self::PixelFormatNotSupported(v) => {
            //     return write!(
            //         f,
            //         "Pixel format {} is not supported. Supported ones: {}. Optimal choice: {}",
            //         v.0, v.1, v.2
            //     );
            // }
            Self::InternalError(err) => return write!(f, "ffmpeg error: {err}"),
        };
        s.fmt(f)
    }
}

impl std::error::Error for FFmpegError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::InternalError(ref e) => Some(e),
            _ => None,
        }
    }
}
