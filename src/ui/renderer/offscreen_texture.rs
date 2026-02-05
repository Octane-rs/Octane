use std::ops::Deref;

use eframe::egui::TextureId;
use eframe::egui_wgpu::Renderer;

use crate::ui::renderer::create_2d_texture;
use crate::ui::renderer::frame::convert::TEXTURE_FORMAT;

/// Managed texture decoupled from the egui rendering pipeline.
pub struct OffscreenTexture {
    id: TextureId,
    label: String,
    /// Raw texture
    pub texture: wgpu::Texture,
    /// Texture view
    pub view: wgpu::TextureView,
}

impl OffscreenTexture {
    pub fn new(
        device: &wgpu::Device,
        renderer: &mut Renderer,
        width: u32,
        height: u32,
        label: &str,
    ) -> Self {
        let texture = create_2d_texture(
            device,
            width,
            height,
            TEXTURE_FORMAT,
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let id = renderer.register_native_texture(device, &view, wgpu::FilterMode::Linear);

        // todo texture cleanup

        Self {
            id,
            label: label.to_owned(),
            texture,
            view,
        }
    }

    /// Texture id
    pub const fn id(&self) -> TextureId {
        self.id
    }

    /// Resize the texture.
    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        renderer: &mut Renderer,
        width: u32,
        height: u32,
    ) {
        if self.texture.width() == width && self.texture.height() == height {
            return;
        }

        *self = Self::new(device, renderer, width, height, &self.label);
    }
}

impl Deref for OffscreenTexture {
    type Target = wgpu::Texture;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}
