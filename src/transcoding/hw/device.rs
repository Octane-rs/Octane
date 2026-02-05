use std::ffi::{CStr, CString};
use std::ptr;

use ffmpeg_next::{codec, ffi};

use crate::transcoding::error::FFmpegError;
use crate::transcoding::ffmpeg_result;
use crate::transcoding::hw::device_index::HWDeviceIndex;

/// Instantiated hardware device for video transcoding
#[derive(Debug)]
pub struct HWDevice {
    /// Physical interface index
    index: HWDeviceIndex,
    /// Device reference. Non-null pointer
    device_ref: *mut ffi::AVBufferRef,
    /// Hardware device type
    device_type: ffi::AVHWDeviceType,

    /// Supported hardware formats. Non-empty.
    pub hw_formats: Vec<ffi::AVPixelFormat>,
    /// Supported software formats. Non-empty.
    pub sw_formats: Vec<ffi::AVPixelFormat>,
    /// Minimum size supported (width, height)
    pub min_size: (i32, i32),
    /// Maximum size supported (width, height)
    pub max_size: (i32, i32),
}

unsafe impl Sync for HWDevice {}
unsafe impl Send for HWDevice {}

impl HWDevice {
    /// Creates a new hardware device
    pub fn new(
        index: HWDeviceIndex,
        device_type: ffi::AVHWDeviceType,
    ) -> Result<Self, FFmpegError> {
        let mut device_ref = ptr::null_mut();

        ffmpeg_result!({
            let index = match index {
                HWDeviceIndex::Global => c"".to_owned(),
                HWDeviceIndex::Index(i) => CString::new(i.to_string()).expect("Numbers are valid C string")
            };
            ffi::av_hwdevice_ctx_create(
                &raw mut device_ref,
                device_type,
                index.as_ptr(),
                ptr::null_mut(),
                0,
            )}; HWDeviceCreateError);
        assert!(!device_ref.is_null(), "Device reference is null");

        let mut constraints_ref =
            unsafe { ffi::av_hwdevice_get_hwframe_constraints(device_ref, ptr::null()) };
        if constraints_ref.is_null() {
            return Err(FFmpegError::HWDeviceConstraintsUnavailable);
        }

        let device = Self {
            index,
            device_ref,
            device_type,
            hw_formats: unsafe {
                pixel_format_list_to_vec((*constraints_ref).valid_hw_formats, false)
            },
            sw_formats: unsafe {
                pixel_format_list_to_vec((*constraints_ref).valid_sw_formats, false)
            },
            min_size: unsafe { ((*constraints_ref).min_width, (*constraints_ref).min_height) },
            max_size: unsafe { ((*constraints_ref).max_width, (*constraints_ref).max_height) },
        };
        let _: () = unsafe { ffi::av_hwframe_constraints_free(&raw mut constraints_ref) };

        assert!(
            !device.hw_formats.is_empty(),
            "No hardware formats found for device: {device:?}"
        );
        assert!(
            !device.sw_formats.is_empty(),
            "No software formats found for device: {device:?}"
        );

        Ok(device)
    }

    /// Attaches the hardware device to a transcoder context using `hw_frames_ctx`.
    ///
    /// # Arguments
    ///
    /// - `ctx`: The transcoder.
    /// - `pixel_format`: The sw pixel format.
    /// - `size`: The size of video to process in pixels (width, height).
    pub fn bind_transcoder(
        &self,
        ctx: &mut codec::context::Context,
        size: (i32, i32),
    ) -> Result<(), FFmpegError> {
        // todo: choose hw format ?
        let hw_pixel_format = *self.hw_formats.first().expect("HW formats is non empty");

        unsafe {
            let ctx_ref = ctx.as_mut_ptr();

            // todo: choose sw format ?
            // let sw_pixel_format = ffi::avcodec_default_get_format(ctx_ref, &hw_pixel_format);
            let sw_pixel_format = ffi::AVPixelFormat::AV_PIX_FMT_NV12;

            if (*ctx_ref).hw_frames_ctx.is_null() {
                let device_ref = ffi::av_buffer_ref(self.device_ref);
                let mut frames_ctx_ref = ffi::av_hwframe_ctx_alloc(device_ref);
                if frames_ctx_ref.is_null() {
                    return Err(FFmpegError::HWFramesContextCreationError);
                }

                let hw_frames_ctx = (*frames_ctx_ref).data.cast::<ffi::AVHWFramesContext>();

                if (*hw_frames_ctx).format == ffi::AVPixelFormat::AV_PIX_FMT_NONE {
                    (*hw_frames_ctx).format = hw_pixel_format;
                }
                if (*hw_frames_ctx).sw_format == ffi::AVPixelFormat::AV_PIX_FMT_NONE {
                    (*hw_frames_ctx).sw_format = sw_pixel_format;
                }
                (*hw_frames_ctx).width = size.0;
                (*hw_frames_ctx).height = size.1;

                if matches!(
                    self.device_type,
                    ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_QSV
                        | ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_VAAPI
                ) {
                    (*hw_frames_ctx).initial_pool_size = 20;
                }

                // Initialize the hw frames context.
                let err = ffi::av_hwframe_ctx_init(frames_ctx_ref);
                if err < 0 {
                    ffi::av_buffer_unref(&raw mut frames_ctx_ref);

                    return Err(FFmpegError::HWFramesContextCreationError);
                }

                (*ctx_ref).hw_frames_ctx = frames_ctx_ref;
            }

            let frames_ctx_ref = (*ctx_ref).hw_frames_ctx;
            let hw_frames_ctx_ref = (*frames_ctx_ref).data.cast::<ffi::AVHWFramesContext>();

            // if self.device_type == ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_VULKAN {
            //     let vulkan_frames_ctx_ref =
            //         (*hw_frames_ctx_ref).hwctx.cast::<VulkanFramesContext>();
            //
            //     (*vulkan_frames_ctx_ref).tiling = ImageTiling::LINEAR;
            //     (*vulkan_frames_ctx_ref).format = [
            //         vk::Format::R4G4_UNORM_PACK8,
            //         vk::Format::UNDEFINED,
            //         vk::Format::UNDEFINED,
            //         vk::Format::UNDEFINED,
            //         vk::Format::UNDEFINED,
            //         vk::Format::UNDEFINED,
            //         vk::Format::UNDEFINED,
            //         vk::Format::UNDEFINED,
            //     ];
            //     (*vulkan_frames_ctx_ref).flags = VkFrameFlags::None;
            // }

            // info!("frames_ctx.format: {:?}", (*hw_frames_ctx).format);
            // info!("frames_ctx.sw_format: {:?}", (*hw_frames_ctx).sw_format);

            // Apply the parameters to the transcoder context.
            (*ctx_ref).pix_fmt = (*hw_frames_ctx_ref).format;
            if !(*ctx_ref).hw_device_ctx.is_null() {
                ffi::av_buffer_unref(&raw mut (*ctx_ref).hw_device_ctx);
            }
            (*ctx_ref).hw_device_ctx = ffi::av_buffer_ref(self.device_ref);
            (*ctx_ref).width = size.0;
            (*ctx_ref).height = size.1;
        };

        Ok(())
    }

    pub const fn device_index(&self) -> HWDeviceIndex {
        self.index
    }

    pub const fn device_type(&self) -> ffi::AVHWDeviceType {
        self.device_type
    }

    pub fn name(&self) -> String {
        unsafe {
            let name_ptr = ffi::av_hwdevice_get_type_name(self.device_type);
            CStr::from_ptr(name_ptr).to_string_lossy().into()
        }
    }
}

impl Drop for HWDevice {
    fn drop(&mut self) {
        unsafe {
            ffi::av_buffer_unref(&raw mut self.device_ref);
        }
    }
}

/// Reads a list of pixel formats from a pointer into a vector.
///
/// # Arguments
///
/// - `ptr`: Pointer to the list of pixel formats.
/// - `free`: Whether to free `ptr` after reading.
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn pixel_format_list_to_vec(
    ptr: *const ffi::AVPixelFormat,
    free: bool,
) -> Vec<ffi::AVPixelFormat> {
    /// Worst case bounds for the pixel format list.
    const BOUNDS: usize = 250;

    let mut result = Vec::new();
    let mut current = ptr;

    while !current.is_null()
        && *current != ffi::AVPixelFormat::AV_PIX_FMT_NONE
        && result.len() < BOUNDS
    {
        result.push(*current);
        current = current.add(1);
    }

    if free {
        ffi::av_free(ptr.cast_mut().cast());
    }

    result
}
