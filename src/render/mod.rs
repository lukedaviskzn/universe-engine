mod pipeline;
mod mesh;
mod camera;
mod texture;
mod graph;

pub use pipeline::*;
pub use mesh::*;
pub use camera::*;
pub use texture::*;
pub use graph::*;

use std::{marker::PhantomData, mem::size_of, sync::Arc};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: glam::Vec3,
    // pub tex_coords: glam::Vec2,
    // pub normal: glam::Vec3,
    pub colour: glam::Vec3,
}

impl Vertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
    };
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub model: glam::Mat4,
}

impl Instance {
    pub fn new(model: glam::Mat4) -> Self {
        Self {
            model,
        }
    }

    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4],
    };
}

pub struct UniformBuffer<T: bytemuck::Pod + bytemuck::Zeroable> {
    renderer: Arc<Renderer>,
    buffer: wgpu::Buffer,
    _marker: PhantomData<T>,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> UniformBuffer<T> {
    pub fn new(renderer: Arc<Renderer>, uniform: T) -> UniformBuffer<T> {
        let buffer = renderer.0.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        UniformBuffer {
            renderer,
            buffer,
            _marker: PhantomData,
        }
    }

    // pub fn buffer(&self) -> &wgpu::Buffer {
    //     &self.buffer
    // }

    pub fn mutate(&mut self, uniform: T) {
        self.renderer.1.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn bind_group_layout(&self) -> wgpu::BindGroupLayout {
        self.renderer.0.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn bind_group(&self, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        self.renderer.0.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.buffer.as_entire_binding(),
                },
            ],
        })
    }
}

pub struct Renderer(pub wgpu::Device, pub wgpu::Queue);
