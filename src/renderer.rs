use std::{sync::Arc, time::Instant};
use wgpu::util::DeviceExt;
use winit::event::{WindowEvent, MouseScrollDelta, KeyEvent, ElementState};
use winit::keyboard::{Key, NamedKey};

use crate::obj_loader::{Mesh, Vertex};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub time: f32,
    pub zoom: f32,
    pub mode: u32,
    pub _pad: u32,
}

pub struct GpuRenderer<'window> {
    pub surface: wgpu::Surface<'window>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub config: wgpu::SurfaceConfiguration,
    globals_buf: wgpu::Buffer,
    globals_bg: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    start_time: Instant,
    pub zoom: f32,
    pub mode: u32,
}

impl<'window> GpuRenderer<'window> {
    pub async fn new(window: &'window winit::window::Window) -> Self {
        // Instance/Surface/Adapter
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).expect("surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("adapter");

        // Device/Queue (wgpu 0.22 API)
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("device");
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Surface config (wgpu 0.22)
        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Shader externo
        let shader_src = std::fs::read_to_string("src/shader.wgsl")
            .expect("No se pudo leer src/shader.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader.wgsl"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });

        // Uniforms
        let globals = Globals { time: 0.0, zoom: 1.0, mode: 2, _pad: 0 };
        let globals_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("globals"),
            contents: bytemuck::bytes_of(&globals),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("globals_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let globals_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals_bg"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&globals_layout],
            push_constant_ranges: &[],
        });

        // Pipeline (wgpu 0.22 requiere compilation_options y cache)
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            globals_buf,
            globals_bg,
            pipeline,
            start_time: Instant::now(),
            zoom: 1.0,
            mode: 2, // 1=Star, 2=Rock, 3=Gas, 4=Earth, 5=Moon
        }
    }

    pub fn render(&mut self, mesh: &Mesh) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Actualizar uniforms
        let g = Globals {
            time: self.start_time.elapsed().as_secs_f32(),
            zoom: self.zoom,
            mode: self.mode,
            _pad: 0,
        };
        self
            .queue
            .write_buffer(&self.globals_buf, 0, bytemuck::bytes_of(&g));

        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("encoder") });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.03,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.globals_bg, &[]);
            mesh.draw(&mut rpass);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    // Teclas 1–5 y zoom con +/− y scroll (winit 0.30)
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event: KeyEvent { logical_key, state, .. },
                ..
            } => {
                if *state == ElementState::Pressed {
                    if let Key::Character(ch) = logical_key.as_ref() {
                        match ch.as_ref() {
                            "1" => self.mode = 1,
                            "2" => self.mode = 2,
                            "3" => self.mode = 3,
                            "4" => self.mode = 4,
                            "5" => self.mode = 5,
                            "+" => self.zoom *= 0.9,
                            "-" => self.zoom *= 1.1,
                            _ => {}
                        }
                    }
                }
                true
            }

            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        if *y > 0.0 { self.zoom *= 0.9; } else { self.zoom *= 1.1; }
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        if pos.y > 0.0 { self.zoom *= 0.9; } else { self.zoom *= 1.1; }
                    }
                }
                true
            }

            _ => false,
        }
    }

}
