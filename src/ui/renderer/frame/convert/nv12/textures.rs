use eframe::wgpu::{Device, Texture, TextureFormat, TextureUsages};

use crate::ui::renderer::create_2d_texture;

pub struct Nv12Textures {
    y: Texture,
    uv: Texture,
}

impl Nv12Textures {
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        let usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;

        Self {
            y: create_2d_texture(device, width, height, TextureFormat::R8Unorm, usage),
            uv: create_2d_texture(
                device,
                width / 2,
                height / 2,
                TextureFormat::Rg8Unorm,
                usage,
            ),
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

    pub const fn uv(&self) -> &Texture {
        &self.uv
    }
}
