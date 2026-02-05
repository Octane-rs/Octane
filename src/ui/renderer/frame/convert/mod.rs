//! Shaders and color space conversion pipelines

pub mod nv12;
mod utils;
pub mod yuv420p;

use ffmpeg_next::frame;

/// Converter output texture format
pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// Converter output texture format bytes per pixel
pub const TEXTURE_FORMAT_BYTES: u32 = 4;

/// Abstraction for a GPU-based pixel format converter.
/// It manages its own source textures (e.g., Y and UV planes) and pipeline.
pub trait PixelConverter: Send + Sync {
    /// Texture width
    fn width(&self) -> u32;

    /// Texture height
    fn height(&self) -> u32;

    /// Uploads the raw frame data to the GPU source textures.
    fn update(&mut self, queue: &wgpu::Queue, frame: &frame::Video);

    /// Records the draw command to convert source -> target.
    /// The `RenderPass` provided must target the `VideoPlayer`'s output texture.
    fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>);

    /// Resize internal source textures if resolution changes.
    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32);
}
