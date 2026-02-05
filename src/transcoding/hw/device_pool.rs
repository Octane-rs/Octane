use std::sync::LazyLock;

use ffmpeg_next::ffi;

use crate::transcoding::hw::device::HWDevice;
use crate::transcoding::hw::device_index::HWDeviceIndex;
use crate::transcoding::hw::device_type::HWDeviceType;

/// Pool of hardware devices containing only valid hardware devices and ready to use
#[derive(Debug)]
pub struct HWDevicePool {
    global: Box<[HWDevice]>,
    devices: Box<[(u8, Box<[HWDevice]>)]>,
}

impl HWDevicePool {
    /// All the supported hw devices
    pub fn all() -> &'static Self {
        static POOL: LazyLock<HWDevicePool> = LazyLock::new(|| {
            /// Maximum number of devices to check for
            const DEVICES_COUNT: u8 = 4;

            let global = get_devices(HWDeviceIndex::Global);

            let mut interfaces = Vec::new();
            for index in 0..DEVICES_COUNT {
                let devices = get_devices(index.into());
                // Stop as soon as a physical interface contain no devices.
                if devices.is_empty() {
                    break;
                }
                interfaces.push((index, devices));
            }

            HWDevicePool {
                global,
                devices: interfaces.into_boxed_slice(),
            }
        });

        &POOL
    }

    /// Get the first [`HWDevice`] satisfying the device type.
    ///
    /// # Arguments
    ///
    /// - `device_type`: The device type to find.
    pub fn first(device_type: ffi::AVHWDeviceType) -> Option<&'static HWDevice> {
        let find = |d: &&HWDevice| d.device_type() == device_type;

        let all = Self::all();

        all.global.iter().find(find).or_else(|| {
            all.devices
                .iter()
                .find_map(|(_, devices)| devices.iter().find(find))
        })
    }
}

/// Get all hardware devices for a physical interface.
///
/// # Arguments
///
/// - `index`: The index of the physical interface.
fn get_devices(index: HWDeviceIndex) -> Box<[HWDevice]> {
    let device_types = match index {
        HWDeviceIndex::Global => HWDeviceType::all_global(),
        HWDeviceIndex::Index(_) => HWDeviceType::all_indexable(),
    };

    device_types
        .iter()
        .map(|dt| std::thread::spawn(move || HWDevice::new(index, dt.device_type())))
        .filter_map(|handle| handle.join().map_or(None, Result::ok))
        .collect()
}
