use wgpu::util::DeviceExt;

use super::{Renderer, Vertex};

pub struct Mesh {
    pub vertices: (u32, wgpu::Buffer),
    pub indices: Option<(u32, wgpu::Buffer)>,
}

impl Mesh {
    pub fn new(renderer: &Renderer, vertices: &[Vertex]) -> Self {
        Self {
            vertices: (vertices.len() as u32, renderer.0.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })),
            indices: None,
        }
    }

    pub fn with_indices(renderer: &Renderer, vertices: &[Vertex], indices: &[u32]) -> Self {
        Self {
            vertices: (vertices.len() as u32, renderer.0.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })),
            indices: Some((indices.len() as u32, renderer.0.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            }))),
        }
    }
}
