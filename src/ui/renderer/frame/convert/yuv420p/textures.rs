use eframe::wgpu::{Device, Texture, TextureFormat, TextureUsages};

use crate::ui::renderer::create_2d_texture;

pub struct Yuv420pTextures {
    y: Texture,
    u: Texture,
    v: Texture,
}

impl Yuv420pTextures {
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        let usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;

        Self {
            y: create_2d_texture(device, width, height, TextureFormat::R8Unorm, usage),
            u: create_2d_texture(device, width / 2, height / 2, TextureFormat::R8Unorm, usage),
            v: create_2d_texture(device, width / 2, height / 2, TextureFormat::R8Unorm, usage),
        }
    }

    pub fn width(&self) -> u32 {
        self.y.width()
    }

    pub fn height(&self) -> u32 {
        self.y.height()
    }

    pub const fn y(&self) -> &Texture {
        &self.y
    }

    pub const fn u(&self) -> &Texture {
        &self.u
    }

    pub const fn v(&self) -> &Texture {
        &self.v
    }
}
