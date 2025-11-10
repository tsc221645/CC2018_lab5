mod math;
mod obj_loader;
mod renderer;

use std::sync::Arc;
use winit::{
    event::*,
    event_loop::EventLoop,
};

fn main() {
    env_logger::init();

    // Crear loop y ventana (winit 0.30)
    let event_loop = EventLoop::new().expect("crear event loop");
    let window = Arc::new(event_loop.create_window(Default::default()).expect("crear ventana"));
    window.set_title("Lab5 - Planetas (wgpu 0.22 / winit 0.30)");

    // ⬇️ Alarga la vida del clone para evitar E0716
    let window_for_gpu = window.clone();
    let mut gpu = pollster::block_on(renderer::GpuRenderer::new(&window_for_gpu));

    // Malla (usa tu loader si quieres)
    let mesh = obj_loader::make_uv_sphere(&gpu.device, &gpu.queue, 64, 64);

    // Otra referencia para el closure (request_redraw)
    let window_ref = window.clone();

    // Bucle principal (run está deprecado, pero funciona)
    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, .. } => {
                    if gpu.input(&event) {
                        return;
                    }
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(size) => gpu.resize(size),
                        WindowEvent::RedrawRequested => {
                            let _ = gpu.render(&mesh);
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => window_ref.request_redraw(),
                _ => {}
            }
        })
        .unwrap();
}