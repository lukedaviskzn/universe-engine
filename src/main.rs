use std::{collections::HashMap, fs::File, io::Write, sync::{mpsc, Arc}, time::Instant};

use clap::Parser;
use flate2::write::GzEncoder;
use fp::Vec3F;
use wgpu::util::DeviceExt;
use winit::{event::{ElementState, Event, KeyEvent, WindowEvent}, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowBuilder}};

#[macro_use]
extern crate fixed_macro;
#[macro_use]
extern crate static_assertions;
#[macro_use]
extern crate maplit;

mod fp;
mod transform;
mod tree;
mod universe;
mod render;

struct StarBuffer {
    centre: Vec3F,
    model_uniform: render::UniformBuffer<glam::Mat4>,
    model_bind_group: wgpu::BindGroup,
    mesh: Arc<render::Mesh>,
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    renderer: Arc<render::Renderer>,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
    frame_count: usize,
    main_pipeline: render::Pipeline,
    tonemap_pipeline: render::Pipeline,
    // postprocess_queue: render::PostprocessQueue,
    render_graph: Option<render::RenderGraph>,
    hdr_buffer: (render::Texture, wgpu::BindGroupLayout, wgpu::BindGroup),
    depth: render::Texture,
    camera: render::Camera,
    camera_uniform: render::UniformBuffer<glam::Mat4>,
    camera_bind_group: wgpu::BindGroup,
    fovy_factor: render::UniformBuffer<f32>,
    fovy_factor_bind_group: wgpu::BindGroup,
    vis_rx: mpsc::Receiver<Vec<StarBuffer>>,
    vis_tx: Option<mpsc::Sender<(Vec3F, f32, u32)>>,
    vis_handle: Option<std::thread::JoinHandle<()>>,
    star_buffers: Vec<StarBuffer>,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window, mut universe: universe::Universe) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let renderer = {
            let (device, queue) = adapter.request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            ).await.unwrap();
            Arc::new(render::Renderer(device, queue))
        };

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            // present_mode: wgpu::PresentMode::Immediate,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let depth = render::Texture::new_depth(&renderer, size.width, size.height);

        let camera = render::Camera::new(transform::Transform::with_translation(Vec3F::from_f64s(1.543e+11, 0.0, 1.0e17)), std::f32::consts::FRAC_PI_2);
        let camera_uniform = render::UniformBuffer::new(Arc::clone(&renderer), camera.perspective(1.0));

        let camera_layout = camera_uniform.bind_group_layout();
        let camera_bind_group = camera_uniform.bind_group(&camera_layout);

        let fovy_factor = render::UniformBuffer::new(Arc::clone(&renderer), 1.0);
        let rads_per_pixel_layout = fovy_factor.bind_group_layout();
        let fovy_factor_bind_group = fovy_factor.bind_group(&rads_per_pixel_layout);

        let model = render::UniformBuffer::new(Arc::clone(&renderer), glam::Mat4::IDENTITY);
        let model_layout = model.bind_group_layout();

        let hdr_buffer = {
            let hdr_texture = render::Texture::new_hdr(&renderer, size.width, size.height);
    
            let hdr_layout = renderer.0.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    }
                ],
            });
    
            let hdr_bind_group = renderer.0.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &hdr_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&hdr_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&hdr_texture.sampler),
                    },
                ],
            });

            (hdr_texture, hdr_layout, hdr_bind_group)
        };

        let main_pipeline = render::Pipeline::new(Arc::clone(&renderer), &wgsl_preprocessor::preprocess!("shaders/shader.wgsl").0, wgpu::PrimitiveTopology::PointList, render::Texture::HDR_FORMAT, true, &[render::Vertex::LAYOUT, render::Instance::LAYOUT], &[&camera_layout, &rads_per_pixel_layout, &model_layout], render::BlendMode::Add).unwrap();
        let tonemap_pipeline = render::Pipeline::new(Arc::clone(&renderer), &wgsl_preprocessor::preprocess!("shaders/postprocess/tonemap.wgsl").0, wgpu::PrimitiveTopology::TriangleStrip, config.format, false, &[], &[&hdr_buffer.1], render::BlendMode::Normal).unwrap();

        let render_graph = {
            let blur_x_source = wgsl_preprocessor::preprocess_with("shaders/postprocess/gaussian_blur.wgsl", hashmap! {
                "BLUR_DIR_X".into() => "1.0".into(),
                "BLUR_DIR_Y".into() => "0.0".into(),
            }).unwrap().0;
            let blur_y_source = wgsl_preprocessor::preprocess_with("shaders/postprocess/gaussian_blur.wgsl", hashmap! {
                "BLUR_DIR_X".into() => "0.0".into(),
                "BLUR_DIR_Y".into() => "1.0".into(),
            }).unwrap().0;

            let bloom_down_source = wgsl_preprocessor::preprocess!("shaders/postprocess/bloom_threshold.wgsl").0;
            let bloom_recombine_source = wgsl_preprocessor::preprocess!("shaders/postprocess/bloom_recombine.wgsl").0;
            let identity_source = wgsl_preprocessor::preprocess!("shaders/postprocess/identity.wgsl").0;
            let aberration_source = wgsl_preprocessor::preprocess!("shaders/postprocess/chromatic_aberration.wgsl").0;

            let screen_size = glam::uvec2(size.width, size.height);
            
            // let mut queue = render::PostprocessQueue::new(Arc::clone(&renderer));
            let mut graph = render::InGraph::new();

            let hdr = graph.add_node(render::RenderNodeDesc {
                label: Some("identity".into()),
                source: identity_source.clone(),
                size_ratio: 1.0,
            });

            let abberation = graph.add_node(render::RenderNodeDesc {
                label: Some("aberration".into()),
                source: aberration_source,
                size_ratio: 1.0,
            });

            let recombine = graph.add_node(render::RenderNodeDesc {
                label: Some("bloom_recombine".into()),
                source: bloom_recombine_source,
                size_ratio: 1.0,
            });

            graph.add_edge(hdr, recombine, ());
            graph.add_edge(recombine, abberation, ());

            // hdr ─> threshold ─> blur (1/2) ─> threshold ─> blur (1/4) ─> threshold ─> blur (1/8) ─> threshold ─> blur (1/16) ─> threshold ─> blur (1/32)
            // recombine <──────────┴──────────────────────────┴──────────────────────────┴──────────────────────────┴───────────────────────────┘
            //  └─> aberration

            let mut prev_pass = hdr;
            
            for i in 0..5 {
                let size_ratio = 2.0f32.powi(-i);
                
                let down = graph.add_node(render::RenderNodeDesc {
                    label: Some(format!("bloom_threshold_{i}").into()),
                    source: bloom_down_source.clone(),
                    size_ratio,
                });
                let blur_x = graph.add_node(render::RenderNodeDesc {
                    label: Some(format!("blur_x_{i}").into()),
                    source: blur_x_source.clone(),
                    size_ratio,
                });
                let blur_y = graph.add_node(render::RenderNodeDesc {
                    label: Some(format!("blur_y_{i}").into()),
                    source: blur_y_source.clone(),
                    size_ratio,
                });

                graph.add_edge(prev_pass, down, ());
                
                graph.add_edge(down, blur_x, ());
                graph.add_edge(blur_x, blur_y, ());
                graph.add_edge(blur_y, recombine, ());

                prev_pass = blur_y;
            }

            render::RenderGraph::compile(graph, Arc::clone(&renderer), screen_size, &hdr_buffer.0)
        };

        let (tx, vis_rx) = mpsc::channel();
        let (vis_tx, rx) = mpsc::channel();

        let vis_handle = {
            let renderer = Arc::clone(&renderer);

            std::thread::spawn(move || {
                let mut star_cache = HashMap::new();

                'outer: loop {
                    let mut camera_pos = None;
                    // skip extras camera_positions if present in buffer
                    loop {
                        match rx.try_recv() {
                            Ok(p) => camera_pos = Some(p),
                            Err(mpsc::TryRecvError::Disconnected) => break 'outer,
                            Err(mpsc::TryRecvError::Empty) => if camera_pos.is_some() {
                                break;
                            } else {
                                let Ok((cam_pos, fovy, screen_height)) = rx.recv() else { break 'outer };
                                camera_pos = Some((cam_pos, fovy, screen_height));
                            },
                        }
                    }
                    let (camera_pos, fovy, screen_height) = camera_pos.expect("unreachable");

                    let visible = universe.all_visible_from(camera_pos, fovy, screen_height);
        
                    for (fresh, _, _) in star_cache.values_mut() {
                        *fresh = false;
                    }

                    let num_bodies: usize = visible.iter().map(|c| c.bodies.iter().map(|b| b.is_body as usize)).flatten().sum();
                    let total: usize = visible.iter().map(|c| c.bodies.len()).sum();
            
                    for cell_v in visible {
                        // if stars already cached, just update model matrix
                        if let Some((fresh, _, _)) = star_cache.get_mut(&cell_v) {
                            *fresh = true;
                            continue;
                        }

                        let pos = cell_v.centre;
            
                        let vertices = cell_v.bodies.iter().map(|tree::PointLight { position, colour, .. }| {
                            render::Vertex {
                                position: (*position - cell_v.centre).to_vec3(),
                                colour: (*colour / 1.0e8).as_vec3(), // scale down to prevent overflow
                            }
                        }).collect::<Vec<_>>();

                        star_cache.insert(cell_v, (true, pos, Arc::new(render::Mesh::new(&renderer, &vertices))));
                    }
                    
                    {
                        let mut old = vec![];
                        
                        for (k, (fresh, _, _)) in &star_cache {
                            if !fresh {
                                old.push(k.clone());
                            }
                        }
            
                        for k in old {
                            star_cache.remove(&k);
                        }
                    }

                    log::debug!("calculated visibility: {num_bodies} bodies, {} point approx, {total} total", total - num_bodies);

                    let mut v = vec![];

                    for (_, pos, mesh) in star_cache.values() {
                        let model = render::UniformBuffer::new(Arc::clone(&renderer), glam::Mat4::from_translation((*pos - camera_pos).to_vec3()));
                        let bind_group = model.bind_group(&model_layout);
                        
                        v.push(StarBuffer {
                            centre: *pos,
                            model_uniform: model,
                            model_bind_group: bind_group,
                            mesh: Arc::clone(&mesh),
                        });
                    }

                    v.sort_by_key(|b| -((b.centre - camera_pos).to_dvec3().length() / 1.0e9) as i128);

                    if let Err(_) = tx.send(v) {
                        break;
                    }
                }
            })
        };

        Self {
            surface,
            renderer,
            config,
            size,
            window,
            frame_count: 0,
            main_pipeline,
            tonemap_pipeline,
            // postprocess_queue,
            render_graph: Some(render_graph),
            hdr_buffer,
            depth,
            camera,
            camera_uniform,
            camera_bind_group,
            fovy_factor,
            fovy_factor_bind_group,
            vis_rx,
            vis_tx: Some(vis_tx),
            vis_handle: Some(vis_handle),
            star_buffers: vec![],
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.renderer.0, &self.config);
            self.depth = render::Texture::new_depth(&self.renderer, self.size.width, self.size.height);
       
            self.hdr_buffer.0 = render::Texture::new_hdr(&self.renderer, self.size.width, self.size.height);
            self.hdr_buffer.2 = self.renderer.0.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.hdr_buffer.1,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.hdr_buffer.0.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.hdr_buffer.0.sampler),
                    },
                ],
            });

            self.render_graph = self.render_graph.take().map(|g| 
                g.resize(glam::uvec2(self.size.width, self.size.height), &self.hdr_buffer.0)
            );
        }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        self.camera.transform.translation -= Vec3F::Z * 1.543e+11;
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.camera_uniform.mutate(self.camera.perspective(self.size.width as f32 / self.size.height as f32));
        
        let fovy_factor = self.size.height as f32 / self.camera.fovy * std::f32::consts::FRAC_PI_2 / 600.0;
        self.fovy_factor.mutate(fovy_factor);

        {
            // send camera position to visibility thread, return immediately if visibility thread shutting down
            let Some(tx) = self.vis_tx.as_ref() else { return Ok(()); };
            let Ok(_) = tx.send((self.camera.transform.translation, self.camera.fovy, self.size.height)) else { return Ok(()); };
        }

        if let Ok(v) = self.vis_rx.try_recv() {
            self.star_buffers = v;
        }

        // update positions relative to camera
        self.star_buffers.iter_mut().for_each(|b| {
            b.model_uniform.mutate(glam::Mat4::from_translation((b.centre - self.camera.transform.translation).to_vec3()));
        });

        let instance_count = 1;
        let instance_buffer = self.renderer.0.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[render::Instance::new(glam::Mat4::IDENTITY)]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut encoder = self.renderer.0.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_buffer.0.view,
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.main_pipeline.0);

            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.fovy_factor_bind_group, &[]);

            for StarBuffer { model_bind_group, mesh, ..  } in &self.star_buffers {
                render_pass.set_vertex_buffer(0, mesh.vertices.1.slice(..));
                render_pass.set_bind_group(2, model_bind_group, &[]);
                render_pass.draw(0..mesh.vertices.0, 0..instance_count);
            }
        }

        let Some(final_bind_group) = self.render_graph.as_ref().map(|g| g.render(&mut encoder)) else { panic!("lost render graph") };
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Tonemap Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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

            render_pass.set_pipeline(&self.tonemap_pipeline.0);

            render_pass.set_bind_group(0, &final_bind_group, &[]);
            // render_pass.set_bind_group(0, &self.hdr_buffer.2, &[]);
            render_pass.draw(0..4, 0..1);
        }

        self.renderer.1.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    EncodeCatalogue {
        #[arg(name="TYPE")]
        cat_type: CatalogueType,
        #[arg(help="input .csv file (see data/catalogue_csv.md for format)")]
        file_in: String,
        #[arg(help="output .bin catalogue file")]
        file_out: String,
    }
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum CatalogueType {
    Stars,
}

async fn run() -> anyhow::Result<()> {
    let Args { command } = Args::parse();

    if let Some(command) = command {
        return run_command(command);
    }
    
    // print!("loading root cell... ");
    // let start = Instant::now();
    // let cell = bincode::deserialize_from::<_, Cell>(GzDecoder::new(File::open("data/cells/cell_7.bin.gz").unwrap())).unwrap();
    // println!("done ({:?})", Instant::now() - start);

    let universe = universe::Universe::new()?;

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window, universe).await;

    event_loop.run(move |event, event_loop| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event: KeyEvent { physical_key: PhysicalKey::Code(KeyCode::Escape), state: ElementState::Pressed, .. },
                        ..
                    } => {
                        // finish vis thread first
                        drop(state.vis_tx.take());
                        state.vis_handle.take().map(|t| t.join());
                        event_loop.exit();
                        return;
                    },
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    // WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    //     state.resize(**new_inner_size);
                    // }
                    WindowEvent::RedrawRequested => {
                        let start = Instant::now();

                        state.update();
                        
                        let ut = Instant::now() - start;
                        let s = Instant::now();
                        
                        match state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                            Err(e) => log::error!("{e:?}"),
                        }

                        let rt = Instant::now() - s;
                        let dt = Instant::now() - start;

                        if state.frame_count % 60 == 0 {
                            log::debug!("FPS {:?}, DT: {:?}, UT: {:?}, RT: {:?}", 1.0 / dt.as_secs_f32(), dt, ut, rt);
                        }

                        state.window().request_redraw();

                        state.frame_count += 1;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }).unwrap();

    Ok(())
}

fn run_command(command: Command) -> anyhow::Result<()> {
    match command {
        Command::EncodeCatalogue { cat_type, file_in, file_out } => {
            if !file_in.ends_with(".csv") {
                return Err(anyhow::anyhow!("Input file path should end with `.csv`."));
            }
            if !file_out.ends_with(".bin.gz") {
                return Err(anyhow::anyhow!("Output file path should end with `.bin.gz`."));
            }
            match cat_type {
                CatalogueType::Stars => {
                    eprint!("reading csv...");
                    let reader = csv::Reader::from_reader(
                        File::open(file_in)?
                    );

                    let catalogue = universe::StarCatalogue::from_csv(reader)?;
                    eprintln!("done");

                    eprint!("encoding...");
                    let data = bincode::serialize(&catalogue)?;
                    eprintln!("done");

                    eprint!("compressing...");
                    GzEncoder::new(File::create(file_out)?, Default::default()).write_all(&data)?;
                    eprintln!("done");
                },
            }
        },
    }

    Ok(())
}

fn main() {
    env_logger::init();
    
    std::panic::set_hook(Box::new(|info| {
        log::error!("{info}");
    }));

    if let Err(err) = pollster::block_on(run()) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
