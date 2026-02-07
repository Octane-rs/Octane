#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    rust_2018_idioms
)]
#![allow(clippy::multiple_crate_versions)]
#![deny(unsafe_code)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate tracing as _;

mod core;
mod services;
mod shell;
mod tracing;
mod transcoding;
mod ui;
mod utils;

use std::io;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use ::tracing::level_filters::LevelFilter;
use eframe::egui::ViewportBuilder;
use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use eframe::wgpu::{
    self, BackendOptions, Backends, InstanceDescriptor, InstanceFlags, PowerPreference, PresentMode,
};
use eframe::{HardwareAcceleration, Renderer};
use scrcpy_launcher::config::ADB_BIN;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Notify;
use wgpu::{ExperimentalFeatures, MemoryBudgetThresholds, Trace};

use crate::shell::app::Octane;
use crate::tracing::init_with_default_level;
use crate::utils::fs::clean_path;

const APP_NAME: &str = env!("CARGO_CRATE_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const ICON: &[u8] = include_bytes!("../assets/icon.png");

#[cfg(target_os = "windows")]
const ADB_PATH: &str = "bin/adb.exe";
#[cfg(not(target_os = "windows"))]
const ADB_PATH: &str = "bin/adb";

pub const HW_DEVICE_INDEX: usize = 0;

fn main() -> Result<(), anyhow::Error> {
    init_with_default_level(LevelFilter::INFO);

    ffmpeg_next::init()?;

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    // builder.worker_threads(4);
    // builder.thread_stack_size();
    let runtime = builder.build()?;
    let _guard = runtime.enter();

    // Adb bin
    runtime.block_on(create_adb_binary(ADB_PATH))?;

    // Static app name
    let app_full_name = &*Box::leak(format!("{APP_NAME} v{APP_VERSION}").into_boxed_str());
    let exit = Arc::new(Notify::new());

    // Application
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([400.0, 600.0])
            .with_min_inner_size([400.0, 400.0])
            .with_icon(eframe::icon_data::from_png_bytes(ICON).expect("Failed to load icon")),
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        hardware_acceleration: HardwareAcceleration::Required,
        renderer: Renderer::Wgpu,
        run_and_return: true,
        event_loop_builder: Some(Box::new(|_builder| {})),
        window_builder: None,
        centered: true,
        wgpu_options: WgpuConfiguration {
            present_mode: PresentMode::Mailbox,
            desired_maximum_frame_latency: Some(1),
            wgpu_setup: WgpuSetup::CreateNew(WgpuSetupCreateNew {
                instance_descriptor: InstanceDescriptor {
                    backends: Backends::VULKAN,
                    flags: InstanceFlags::default(),
                    memory_budget_thresholds: MemoryBudgetThresholds::default(),
                    backend_options: BackendOptions::default(),
                },
                power_preference: PowerPreference::HighPerformance,
                native_adapter_selector: None,
                device_descriptor: Arc::new(|adapter| {
                    let base_limits = if adapter.get_info().backend == wgpu::Backend::Gl {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    };

                    wgpu::DeviceDescriptor {
                        label: Some(app_full_name),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits {
                            // When using a depth buffer, we have to be able to create a texture
                            // large enough for the entire surface, and we want to support 4k+ displays.
                            max_texture_dimension_2d: 8192,
                            ..base_limits
                        },
                        experimental_features: ExperimentalFeatures::default(),
                        memory_hints: wgpu::MemoryHints::Performance,
                        trace: Trace::Off,
                    }
                }),
            }),
            ..Default::default()
        },
        persist_window: true,
        persistence_path: None,
        dithering: true,
    };

    let app_exit = exit.clone();
    let exit_notification = exit.notified();

    let result = eframe::run_native(
        app_full_name,
        native_options,
        Box::new(|cc| Ok(Box::new(Octane::new(cc, app_exit)))),
    );

    runtime.block_on(exit_notification);

    if let Err(err) = result {
        error!("App ended with error: {err}");
    } else {
        info!("App ended");
    }

    let _ = io::stdout().flush();

    Ok(())
}

async fn create_adb_binary(adb_path: impl AsRef<Path>) -> Result<(), io::Error> {
    use tokio::fs::File;

    let adb_path = adb_path.as_ref();

    if fs::try_exists(adb_path).await.unwrap_or_default() {
        return Ok(());
    }

    clean_path(adb_path).await?;

    if let Some(parent) = adb_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let mut options = File::options();
    options.write(true).create_new(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o755);
    }

    let mut file = options.open(adb_path).await?;
    file.write_all(ADB_BIN).await?;
    file.sync_all().await?;

    Ok(())
}
