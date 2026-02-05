/// Writes a texture to a queue
pub fn write_texture(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    data: &[u8],
    stride: u32,
    width: u32,
    height: u32,
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(stride),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

/// Creates a sampler with linear filtering and clamp-to-edge addressing mode
pub fn create_sampler(device: &wgpu::Device, label: Option<&'static str>) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    })
}

/// Creates a bind group layout.
///
/// # Important
///
/// Specify `ENTRIES + 1` for the number of texture entries, as the last entry is reserved for the sampler.
///
/// # Arguments
///
/// - `device` - The `WGpu` device used to create the bind group layout.
pub fn create_bind_group_layout<const ENTRIES: usize>(
    device: &wgpu::Device,
    label: Option<&'static str>,
) -> wgpu::BindGroupLayout {
    let mut entries = [wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }; ENTRIES];

    #[allow(clippy::cast_possible_truncation)]
    entries.iter_mut().enumerate().for_each(|(i, entry)| {
        entry.binding = i as u32;
    });
    entries[ENTRIES - 1].ty = wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering);

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label,
        entries: &entries,
    })
}
