//! High level video rendering interface

use eframe::egui;
use eframe::egui_wgpu::RenderState;
use ffmpeg_next::frame;
use ffmpeg_next::util::format;

use crate::ui::renderer::frame::convert::PixelConverter;
use crate::ui::renderer::frame::convert::nv12::Nv12Converter;
use crate::ui::renderer::frame::convert::yuv420p::Yuv420pConverter;
use crate::ui::renderer::offscreen_texture::OffscreenTexture;

/// Video player, automatically handle size and format changes
pub struct VideoPlayer {
    state: RenderState,
    /// RGBA displayed texture
    texture: Option<OffscreenTexture>,
    /// Abstract pixel converter
    converter: Option<Box<dyn PixelConverter>>,
    /// Current pixel format
    current_format: Option<format::Pixel>,
    /// Last runtime error
    last_error: Option<String>,
}

impl VideoPlayer {
    pub fn new(state: RenderState) -> Self {
        Self {
            state,
            texture: None,
            converter: None,
            current_format: None,
            last_error: None,
        }
    }

    /// Called every frame with new data
    pub fn update(&mut self, frame: &frame::Video) {
        let width = frame.width();
        let height = frame.height();
        let format = frame.format();

        // let planes = frame.planes();
        // let (size, stride) = if frame.planes() > 0 {
        //     (Some(frame.data(0).len()), Some(frame.stride(0)))
        // } else {
        //     (None, None)
        // };
        // let quality = frame.quality();
        // let kind = frame.kind();
        // let color_space = frame.color_space();
        // let color_primaries = frame.color_primaries();
        // let color_range = frame.color_range();

        // println!(
        //     "Frame: {width}x{height} size: {size:?} {format:?} planes: {} stride: {stride:?} quality: {quality:?} {kind:?} Color(space: {color_space:?}, primaries: {color_primaries:?}, range: {color_range:?})",
        //     frame.planes(),
        // );

        if width == 0 || height == 0 {
            return;
        }

        self.last_error.take();

        if let Err(err) = self.ensure_resources(width, height, format) {
            self.last_error.replace(err);
            return;
        }

        let device = &self.state.device;
        let queue = &self.state.queue;

        let texture = self
            .texture
            .as_mut()
            .expect("We ensured resources are created");
        let converter = self
            .converter
            .as_mut()
            .expect("We ensured resources are created");

        converter.update(queue, frame);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Color Conversion Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            converter.render(&mut pass);
        }

        queue.submit(Some(encoder.finish()));
    }

    /// Handles creation and resizing of GPU resources.
    fn ensure_resources(
        &mut self,
        width: u32,
        height: u32,
        format: format::Pixel,
    ) -> Result<(), String> {
        let device = &self.state.device;
        let renderer = &mut *self.state.renderer.write();

        if let Some(texture) = self.texture.as_mut() {
            if texture.width() != width || texture.height() != height {
                texture.resize(device, renderer, width, height);
            }
        } else {
            let texture =
                OffscreenTexture::new(device, renderer, width, height, ">> Video Player Texture");
            self.texture.replace(texture);
        }

        let format_changed = self.current_format != Some(format);

        if !format_changed && let Some(converter) = self.converter.as_mut() {
            if converter.width() != width || converter.height() != height {
                converter.resize(device, width, height);
            }
        } else {
            let new_converter = match format {
                format::Pixel::VULKAN | format::Pixel::NV12 => {
                    Box::new(Nv12Converter::new(device, width, height)) as _
                }
                format::Pixel::YUV420P => {
                    Box::new(Yuv420pConverter::new(device, width, height)) as _
                }
                f => return Err(format!("Unsupported pixel format: {f:?}")),
            };
            self.current_format.replace(format);
            self.converter.replace(new_converter);
        }

        Ok(())
    }
}

impl VideoPlayer {
    pub fn ui(&self, ui: &mut egui::Ui) {
        let Some(texture) = &self.texture else {
            return;
        };

        let available_size = ui.available_size();

        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let video_width = texture.width() as f32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let video_height = texture.height() as f32;

        if video_width == 0.0 || video_height == 0.0 {
            return;
        }

        let video_ratio = video_width / video_height;
        let window_ratio = available_size.x / available_size.y;

        let target_size = if window_ratio > video_ratio {
            egui::vec2(available_size.y * video_ratio, available_size.y)
        } else {
            egui::vec2(available_size.x, available_size.x / video_ratio)
        };

        let x_offset = (available_size.x - target_size.x) / 2.0;
        let y_offset = (available_size.y - target_size.y) / 2.0;

        let (rect, _) = ui.allocate_exact_size(available_size, egui::Sense::hover());

        let image_rect =
            egui::Rect::from_min_size(rect.min + egui::vec2(x_offset, y_offset), target_size);

        ui.painter().image(
            texture.id(),
            image_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
}
