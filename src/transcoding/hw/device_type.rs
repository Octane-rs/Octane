use std::ffi::CStr;
use std::fmt;
use std::sync::LazyLock;

use ffmpeg_next::ffi;

/// Hardware device type with its display name
#[derive(Debug, Copy, Clone)]
pub struct HWDeviceType {
    /// Hardware device type
    device_type: ffi::AVHWDeviceType,
    /// Hardware device type display name
    display_name: &'static str,
}

impl HWDeviceType {
    /// Indexable hardware device types.
    /// Opposed to global hardware device types, these can target specific physical interface using an index.
    const INDEXABLE: &'static [ffi::AVHWDeviceType] = &[
        ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_CUDA,
        ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_DRM,
        ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_VAAPI,
        ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_VULKAN,
    ];

    /// Create a new hardware device type
    fn new(device_type: ffi::AVHWDeviceType) -> Self {
        let name_ptr = unsafe { ffi::av_hwdevice_get_type_name(device_type) };

        Self {
            device_type,
            display_name: unsafe { CStr::from_ptr(name_ptr).to_str().unwrap() },
        }
    }

    /// All the supported hw device types
    pub fn all() -> &'static [Self] {
        static TYPES: LazyLock<Box<[HWDeviceType]>> = LazyLock::new(|| {
            let mut hw_device_types = Vec::new();
            let mut next = ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_NONE;
            loop {
                next = unsafe { ffi::av_hwdevice_iterate_types(next) };
                if next == ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_NONE {
                    break hw_device_types.into_boxed_slice();
                }
                #[cfg(target_os = "windows")]
                if next == ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_VAAPI {
                    continue;
                }
                hw_device_types.push(HWDeviceType::new(next));
            }
        });

        &TYPES
    }

    /// All the supported indexable hw device types.
    /// These can target specific physical interface using an index.
    pub fn all_indexable() -> &'static [Self] {
        static TYPES: LazyLock<Box<[HWDeviceType]>> = LazyLock::new(|| {
            HWDeviceType::all()
                .iter()
                .filter(|dt| HWDeviceType::INDEXABLE.contains(&dt.device_type))
                .copied()
                .collect()
        });

        &TYPES
    }

    /// All the supported global hw device types
    pub fn all_global() -> &'static [Self] {
        static TYPES: LazyLock<Box<[HWDeviceType]>> = LazyLock::new(|| {
            HWDeviceType::all()
                .iter()
                .filter(|dt| !HWDeviceType::INDEXABLE.contains(&dt.device_type))
                .copied()
                .collect()
        });

        &TYPES
    }

    /// Hardware device type
    pub const fn device_type(&self) -> ffi::AVHWDeviceType {
        self.device_type
    }

    /// Hardware device type display name
    pub const fn display_name(&self) -> &'static str {
        self.display_name
    }

    /// Whether the device type is available.
    ///
    /// # Arguments
    ///
    /// - `device_type`: The device type to check.
    pub fn is_available(device_type: ffi::AVHWDeviceType) -> bool {
        Self::all().iter().any(|dt| dt.device_type == device_type)
    }
}

impl fmt::Display for HWDeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_name.fmt(f)
    }
}
