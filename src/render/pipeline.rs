use std::{io, sync::Arc};

use super::{InGraph, Renderer, Texture};

pub struct Pipeline(pub wgpu::RenderPipeline);

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Replace,
    Add,
}

impl Pipeline {
    pub fn new(renderer: Arc<Renderer>, source: &str, topology: wgpu::PrimitiveTopology, target_format: wgpu::TextureFormat, has_depth: bool, vertex_layouts: &[wgpu::VertexBufferLayout<'static>], bind_group_layouts: &[&wgpu::BindGroupLayout], blend_mode: BlendMode) -> Result<Self, io::Error> {
        let shader = renderer.0.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        let blend = match blend_mode {
            BlendMode::Normal => wgpu::BlendState::ALPHA_BLENDING,
            BlendMode::Replace => wgpu::BlendState::REPLACE,
            BlendMode::Add => wgpu::BlendState {
                color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
                alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
            },
        };

        let render_pipeline_layout =
            renderer.0.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });
        
        let render_pipeline = renderer.0.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: has_depth.then_some(wgpu::DepthStencilState {
                format: super::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        Ok(Pipeline(render_pipeline))
    }

    pub fn new_postprocess(renderer: Arc<Renderer>, source: &str, layouts: &[&wgpu::BindGroupLayout]) -> Result<Self, io::Error> {
        let shader = renderer.0.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        let render_pipeline_layout =
            renderer.0.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: layouts,
                push_constant_ranges: &[],
            });
        
        let render_pipeline = renderer.0.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: super::Texture::HDR_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        });

        Ok(Pipeline(render_pipeline))
    }
}

pub struct RenderNodeDesc {
    pub label: Option<Box<str>>,
    pub source: String,
    pub size_ratio: f32,
}

struct RenderNode {
    label: Option<Box<str>>,
    pipeline: Pipeline,
    size_ratio: f32,
    texture: super::Texture,
    layout: wgpu::BindGroupLayout,
}

struct BoundRenderNode(RenderNode, wgpu::BindGroup);

pub struct RenderGraph {
    renderer: Arc<Renderer>,
    graph: InGraph<BoundRenderNode, ()>,
    root_layout: wgpu::BindGroupLayout,
    root_bind_group: wgpu::BindGroup,
}

impl RenderGraph {
    pub fn compile(desc: InGraph<RenderNodeDesc, ()>, renderer: Arc<Renderer>, screen_size: glam::UVec2, hdr_buffer: &Texture) -> Self {
        let graph = desc.map_nodes(|node, edges| {
            let mut entries = Vec::new();
            for i in 0..edges.len().max(1) {
                let i = i as u32;
                
                entries.push(wgpu::BindGroupLayoutEntry {
                    binding: 2 * i,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                });
                entries.push(wgpu::BindGroupLayoutEntry {
                    binding: 2 * i + 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                });
            }

            let layout = renderer.0.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &entries,
            });
            
            let pipeline = Pipeline::new_postprocess(Arc::clone(&renderer), &node.source, &[&layout]).unwrap();
            let texture = super::Texture::new_hdr(&renderer, (screen_size.x as f32 * node.size_ratio) as u32, (screen_size.y as f32 * node.size_ratio) as u32);

            RenderNode {
                label: node.label,
                pipeline,
                size_ratio: node.size_ratio,
                texture,
                layout,
            }
        }).map_edges(|from, _, _| {
            (Arc::clone(&from.texture.view), Arc::clone(&from.texture.sampler))
        }).map_nodes(|n, edges| {
            let mut entries = Vec::with_capacity(edges.len());

            for (i, (_, (view, sampler))) in edges.into_iter().enumerate() {
                let i = i as u32;
                
                entries.push(wgpu::BindGroupEntry {
                    binding: 2 * i,
                    resource: wgpu::BindingResource::TextureView(view),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 2 * i + 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                });
            }
            
            if edges.len() == 0 {
                entries.push(wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_buffer.view),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&hdr_buffer.sampler),
                });
            }

            let bind_group = renderer.0.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &n.layout,
                entries: &entries,
            });

            BoundRenderNode(n, bind_group)
        }).map_edges(|_,_,_| ());

        let root_layout = renderer.0.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let root = *graph.topo_sort().last().unwrap();
        let root = graph.node(root);

        let root_bind_group = renderer.0.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &root_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&root.0.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&root.0.texture.sampler),
                },
            ],
        });

        Self {
            renderer,
            graph,
            root_layout,
            root_bind_group,
        }
    }
    
    pub fn resize(mut self, screen_size: glam::UVec2, hdr_buffer: &Texture) -> Self {
        self.graph.nodes_mut().into_iter().for_each(|node| {
            let size = screen_size.as_vec2() * node.0.size_ratio;
            let size = glam::uvec2(size.x as u32, size.y as u32).max(glam::UVec2::ONE);
            node.0.texture = super::Texture::new_hdr(&self.renderer, size.x, size.y);
        });

        let graph = self.graph.map_edges(|from, _, _| {
            (Arc::clone(&from.0.texture.view), Arc::clone(&from.0.texture.sampler))
        }).map_nodes(|n, edges| {
            let mut entries = Vec::with_capacity(edges.len());

            for (i, (_, (view, sampler))) in edges.into_iter().enumerate() {
                let i = i as u32;
                
                entries.push(wgpu::BindGroupEntry {
                    binding: 2 * i,
                    resource: wgpu::BindingResource::TextureView(view),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 2 * i + 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                });
            }

            if edges.len() == 0 {
                entries.push(wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&*hdr_buffer.view),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&*hdr_buffer.sampler),
                });
            }

            let bind_group = self.renderer.0.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &n.0.layout,
                entries: &entries,
            });

            BoundRenderNode(n.0, bind_group)
        }).map_edges(|_,_,_| ());

        let root = *graph.topo_sort().last().unwrap();
        let root = graph.node(root);

        let root_bind_group = self.renderer.0.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.root_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&root.0.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&root.0.texture.sampler),
                },
            ],
        });

        RenderGraph {
            renderer: self.renderer,
            graph,
            root_layout: self.root_layout,
            root_bind_group,
        }
    }
    
    pub fn render(&self, encoder: &mut wgpu::CommandEncoder) -> &wgpu::BindGroup {
        let sorted = self.graph.topo_sort();

        for &node_id in &sorted {
            let node = self.graph.node(node_id);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&node.0.label.as_ref().map(|l| format!("{l:?} render pass")).unwrap_or("unnamed post-process render pass".into())),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &node.0.texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
    
            render_pass.set_pipeline(&node.0.pipeline.0);
            render_pass.set_bind_group(0, &node.1, &[]);
            render_pass.draw(0..4, 0..1);
        }

        &self.root_bind_group
    }
}
