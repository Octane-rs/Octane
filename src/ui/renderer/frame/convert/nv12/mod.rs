use eframe::wgpu::include_wgsl;
use ffmpeg_next::frame;
use textures::Nv12Textures;
use wgpu::util::DeviceExt;

use crate::ui::renderer::frame::convert::nv12::vertex::{VERTICES, Vertex};
use crate::ui::renderer::frame::convert::utils::{
    create_bind_group_layout, create_sampler, write_texture,
};
use crate::ui::renderer::frame::convert::{PixelConverter, TEXTURE_FORMAT};
pub mod textures;
mod vertex;

pub struct Nv12Converter {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    textures: Nv12Textures,
    width: u32,
    height: u32,
}

impl Nv12Converter {
    pub const NAME: &'static str = "NV12 Converter";

    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let textures = Nv12Textures::new(device, width, height);

        // Sampler
        let sampler = create_sampler(device, Some(Self::NAME));

        // Bind Group Layout
        let bind_group_layout = create_bind_group_layout::<3>(device, Some(Self::NAME));

        // Bind Group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("NV12 Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &textures
                            .y()
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &textures
                            .uv()
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Pipeline
        let shader = device.create_shader_module(include_wgsl!("rgba.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(Self::NAME),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(Self::NAME),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as _,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(TEXTURE_FORMAT.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(Self::NAME),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            pipeline,
            bind_group,
            vertex_buffer,
            textures,
            width,
            height,
        }
    }
}

impl PixelConverter for Nv12Converter {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn update(&mut self, queue: &wgpu::Queue, frame: &frame::Video) {
        write_texture(
            queue,
            self.textures.y(),
            frame.data(0),
            frame.stride(0) as u32,
            self.width,
            self.height,
        );
        write_texture(
            queue,
            self.textures.uv(),
            frame.data(1),
            frame.stride(1) as u32,
            self.width / 2,
            self.height / 2,
        );
    }

    fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..6, 0..1);
    }

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        *self = Self::new(device, width, height);
    }
}
