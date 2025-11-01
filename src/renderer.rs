// Cargo.toml necesita:
// wgpu = "22.0"
// winit = "0.30"
// bytemuck = { version = "1.14", features = ["derive"] }
// pollster = "0.3"

use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::EventLoop,
    window::Window,
};
use std::sync::Arc;
use crate::obj_loader::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    time: f32,
    shader_mode: u32,
    _padding: [u32; 2],
}

pub struct GpuRenderer<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_indices: u32,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
}

impl<'window> GpuRenderer<'window> {
    pub async fn new(window: &'window Window, vertices: &[Vertex], indices: &[u32]) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        // Buffers
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind group
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("bind_group"),
        });

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            num_indices: indices.len() as u32,
            depth_texture,
            depth_view,
        }
    }

    pub fn render(&mut self, uniforms: Uniforms) -> Result<(), wgpu::SurfaceError> {
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

// Convierte tus triángulos a formato GPU
pub fn triangles_to_gpu_data(triangles: &[Triangle]) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for tri in triangles {
        let start_idx = vertices.len() as u32;

        for v in [&tri.v0, &tri.v1, &tri.v2] {
            vertices.push(Vertex {
                position: [v.pos.x, v.pos.y, v.pos.z],
                normal: [v.normal.x, v.normal.y, v.normal.z],
            });
        }

        indices.push(start_idx);
        indices.push(start_idx + 1);
        indices.push(start_idx + 2);
    }

    (vertices, indices)
}

// Shader inline
const SHADER_SOURCE: &str = r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
    time: f32,
    shader_mode: u32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.view_proj * vec4<f32>(input.position, 1.0);
    output.world_normal = input.normal;
    output.world_pos = input.position;
    return output;
}

fn star_shader(normal: vec3<f32>, time: f32) -> vec3<f32> {
    let noise = fract(sin(dot(normal, vec3<f32>(12.9898, 78.233, 45.164))) * 43758.5453);
    let flare = pow(max(0.0, sin(time * 2.0 + noise * 10.0)), 3.0);
    return vec3<f32>(1.0, 0.8, 0.2) + vec3<f32>(0.5, 0.2, 0.0) * flare;
}

fn rock_shader(pos: vec3<f32>) -> vec3<f32> {
    let noise = fract(sin(dot(pos * 10.0, vec3<f32>(12.9898, 78.233, 45.164))) * 43758.5453);
    return vec3<f32>(0.4, 0.3, 0.25) * (0.7 + noise * 0.3);
}

fn gas_shader(pos: vec3<f32>, time: f32) -> vec3<f32> {
    let band = sin(pos.y * 8.0 + time) * 0.5 + 0.5;
    return mix(vec3<f32>(0.8, 0.6, 0.3), vec3<f32>(0.9, 0.7, 0.4), band);
}

fn earth_shader(pos: vec3<f32>) -> vec3<f32> {
    let noise = fract(sin(dot(pos * 5.0, vec3<f32>(12.9898, 78.233, 45.164))) * 43758.5453);
    let ocean = vec3<f32>(0.1, 0.3, 0.6);
    let land = vec3<f32>(0.2, 0.5, 0.2);
    return mix(ocean, land, step(0.4, noise));
}

fn moon_shader(pos: vec3<f32>) -> vec3<f32> {
    let crater = fract(sin(dot(pos * 20.0, vec3<f32>(12.9898, 78.233, 45.164))) * 43758.5453);
    return vec3<f32>(0.6, 0.6, 0.6) * (0.8 + crater * 0.4);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(input.world_normal);
    let light_dir = normalize(vec3<f32>(1.0, 1.0, -1.0));
    let diffuse = max(0.0, dot(normal, light_dir));
    
    var color: vec3<f32>;
    
    switch uniforms.shader_mode {
        case 0u: { color = star_shader(normal, uniforms.time); }
        case 1u: { color = rock_shader(input.world_pos); }
        case 2u: { color = gas_shader(input.world_pos, uniforms.time); }
        case 3u: { color = earth_shader(input.world_pos); }
        case 4u: { color = moon_shader(input.world_pos); }
        default: { color = vec3<f32>(1.0, 0.0, 1.0); }
    }
    
    color = color * (0.3 + diffuse * 0.7);
    
    return vec4<f32>(color, 1.0);
}
"#;

// Función principal de render (reemplaza tu render() actual)
pub fn render(triangles: &Vec<Triangle>) {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(event_loop.create_window(Window::default_attributes()
        .with_title("Laboratorio 5 - GPU")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 800))
    ).unwrap());

    let (vertices, indices) = triangles_to_gpu_data(triangles);
    
    let mut renderer = pollster::block_on(GpuRenderer::new(&window, &vertices, &indices));
    
    let mut angle_y = 0.0f32;
    let mut angle_x = 0.0f32;
    let mut time = 0.0f32;
    let mut dist = 10.0f32;
    let mut shader_mode = 0u32;

    let window_clone = window.clone();

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state == ElementState::Pressed {
                        match event.physical_key {
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) => elwt.exit(),
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit1) => shader_mode = 0,
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit2) => shader_mode = 1,
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit3) => shader_mode = 2,
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit4) => shader_mode = 3,
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit5) => shader_mode = 4,
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyA) => angle_y -= 0.05,
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyD) => angle_y += 0.05,
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyW) => angle_x -= 0.05,
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyS) => angle_x += 0.05,
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyQ) => dist -= 0.5,
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyE) => dist += 0.5,
                            _ => {}
                        }
                    }
                }
                WindowEvent::RedrawRequested => {
                    time += 0.02;
                    
                    // Matriz de proyección simple
                    let aspect = renderer.size.width as f32 / renderer.size.height as f32;
                    let fov: f32 = 1.2;
                    let near: f32 = 0.1;
                    let far: f32 = 100.0;
                    
                    let f = 1.0 / (fov / 2.0).tan();
                    let nf = 1.0 / (near - far);
                    
                    let proj = [
                        [f / aspect, 0.0, 0.0, 0.0],
                        [0.0, f, 0.0, 0.0],
                        [0.0, 0.0, (far + near) * nf, -1.0],
                        [0.0, 0.0, 2.0 * far * near * nf, 0.0],
                    ];
                    
                    // Transformación simple (rotación + traslación)
                    let cy = angle_y.cos();
                    let sy = angle_y.sin();
                    let cx = angle_x.cos();
                    let sx = angle_x.sin();
                    
                    let view = [
                        [cy, sy * sx, sy * cx, 0.0],
                        [0.0, cx, -sx, 0.0],
                        [-sy, cy * sx, cy * cx, 0.0],
                        [0.0, 0.0, -dist, 1.0],
                    ];
                    
                    // Multiplica view * proj (simplificado)
                    let mut view_proj = [[0.0; 4]; 4];
                    for i in 0..4 {
                        for j in 0..4 {
                            for k in 0..4 {
                                view_proj[i][j] += view[i][k] * proj[k][j];
                            }
                        }
                    }
                    
                    let uniforms = Uniforms {
                        view_proj,
                        time,
                        shader_mode,
                        _padding: [0, 0],
                    };
                    
                    match renderer.render(uniforms) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {}
                        Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                window_clone.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
