use std::sync::Arc;

use wgpu::util::DeviceExt;

use super::Renderer;

pub struct Texture {
    pub texture: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub sampler: Arc<wgpu::Sampler>,
}

impl Texture {
    pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    
    fn new(
        renderer: &Renderer, size: wgpu::Extent3d, dimension: wgpu::TextureDimension, format: wgpu::TextureFormat, usage: wgpu::TextureUsages, 
        address_mode_u: wgpu::AddressMode, address_mode_v: wgpu::AddressMode, address_mode_w: wgpu::AddressMode, border_colour: Option<wgpu::SamplerBorderColor>,
        mag_filter: wgpu::FilterMode, min_filter: wgpu::FilterMode, mipmap_filter: wgpu::FilterMode,
    ) -> Self {
        let texture = Arc::new(renderer.0.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            // mip_level_count: size.max_mips(dimension),
            mip_level_count: 1,
            sample_count: 1,
            dimension,
            format,
            usage,
            view_formats: &[],
        }));

        let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        let sampler = Arc::new(renderer.0.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u,
            address_mode_v,
            address_mode_w,
            mag_filter,
            min_filter,
            mipmap_filter,
            border_color: border_colour,
            ..Default::default()
        }));

        Self {
            texture,
            view,
            sampler,
        }
    }

    fn with_data(
        renderer: &Renderer, size: wgpu::Extent3d, dimension: wgpu::TextureDimension, format: wgpu::TextureFormat, usage: wgpu::TextureUsages, 
        address_mode_u: wgpu::AddressMode, address_mode_v: wgpu::AddressMode, address_mode_w: wgpu::AddressMode, border_colour: Option<wgpu::SamplerBorderColor>,
        mag_filter: wgpu::FilterMode, min_filter: wgpu::FilterMode, mipmap_filter: wgpu::FilterMode,
        data: &[u8], order: wgpu::util::TextureDataOrder,
    ) -> Self {
        let texture = Arc::new(renderer.0.create_texture_with_data(&renderer.1, &wgpu::TextureDescriptor {
            label: None,
            size,
            // mip_level_count: size.max_mips(dimension),
            mip_level_count: 1,
            sample_count: 1,
            dimension,
            format,
            usage,
            view_formats: &[],
        }, order, data));

        let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        let sampler = Arc::new(renderer.0.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u,
            address_mode_v,
            address_mode_w,
            mag_filter,
            min_filter,
            mipmap_filter,
            border_color: border_colour,
            ..Default::default()
        }));

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn new_empty_2d(renderer: &Renderer, width: u32, height: u32, format: wgpu::TextureFormat, usage: wgpu::TextureUsages) -> Self {
        Self::new(
            renderer, wgpu::Extent3d { width, height, depth_or_array_layers: 1 }, wgpu::TextureDimension::D2, format, usage,
            wgpu::AddressMode::ClampToEdge, wgpu::AddressMode::ClampToEdge, wgpu::AddressMode::ClampToEdge, None,
            wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest,
        )
    }

    pub fn with_data_2d(renderer: &Renderer, width: u32, height: u32, format: wgpu::TextureFormat, usage: wgpu::TextureUsages, data: &[u8]) -> Self {
        Self::with_data(
            renderer, wgpu::Extent3d { width, height, depth_or_array_layers: 1 }, wgpu::TextureDimension::D2, format, usage,
            wgpu::AddressMode::Repeat, wgpu::AddressMode::Repeat, wgpu::AddressMode::Repeat, None,
            wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest,
            data, Default::default(),
        )
    }

    pub fn new_depth(renderer: &Renderer, width: u32, height: u32) -> Self {
        Self::new_empty_2d(renderer, width, height, Self::DEPTH_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
    }

    pub fn new_hdr(renderer: &Renderer, width: u32, height: u32) -> Self {
        Self::new_empty_2d(renderer, width, height, Self::HDR_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
    }
}
