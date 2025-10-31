use crate::math::*;
use crate::obj_loader::*;
use crate::shaders::*;
use minifb::{Key, Window, WindowOptions};
use nalgebra::{Rotation3, Vector3};

pub fn render(triangles: &Vec<Triangle>) {
    let width = 300;
    let height = 300;
    let mut window = Window::new("Laboratorio 5 - Sistema Planetario", width, height, WindowOptions::default()).unwrap();
    let mut buffer = vec![0u32; width * height];
    let mut zbuffer = vec![f32::INFINITY; width * height];

    let mut mode = ShaderMode::Star;
    let mut angle_y = 0.0f32;
    let mut angle_x = 0.0f32;
    let mut time = 0.0f32;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // controles

        if window.is_key_down(Key::Key1) { mode = ShaderMode::Star; }
        if window.is_key_down(Key::Key2) { mode = ShaderMode::Rock; }
        if window.is_key_down(Key::Key3) { mode = ShaderMode::Gas; }

        if window.is_key_down(Key::A) { angle_y -= 0.03; }
        if window.is_key_down(Key::D) { angle_y += 0.03; }
        if window.is_key_down(Key::W) { angle_x -= 0.03; }
        if window.is_key_down(Key::S) { angle_x += 0.03; }

        // rotación
        let rot_y = Rotation3::from_axis_angle(&Vector3::y_axis(), angle_y);
        let rot_x = Rotation3::from_axis_angle(&Vector3::x_axis(), angle_x);
        let rotation = rot_y * rot_x;

        // limpia buffers
        buffer.fill(0);
        zbuffer.fill(f32::INFINITY);

        // renderiza todos los triángulos
        for tri in triangles {
            static mut DIST: f32 = 3.0;
            unsafe {
                if window.is_key_down(Key::Q) { DIST -= 0.05; }
                if window.is_key_down(Key::E) { DIST += 0.05; }
            }

           let mut pts = [Vec3::zeros(); 3];
           let mut norms = [Vec3::zeros(); 3];

            // cámara y proyección
            let camera_pos = Vec3::new(0.0, 0.0, -unsafe { DIST });

            let fov = 1.2; // campo de visión (cuanto más alto, más zoom)
            let aspect = width as f32 / height as f32;

            for (i, v) in [&tri.v0, &tri.v1, &tri.v2].iter().enumerate() {
                let world = rotation * v.pos;
                let view = world - camera_pos;

                // proyección perspectiva
                let z = view.z.max(0.1);
                let px = (view.x / z) * fov * aspect;
                let py = (view.y / z) * fov;

                pts[i] = Vec3::new(px, py, z);
                norms[i] = (rotation * v.normal).normalize();
            }

            draw_triangle_filled(&mut buffer, &mut zbuffer, width, height, pts, norms, mode, time);

        }

        time += 0.02;
        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}

// --------------------------------------------------
// Relleno de triángulos
// --------------------------------------------------

fn draw_triangle_filled(
    buffer: &mut [u32],
    zbuffer: &mut [f32],
    width: usize,
    height: usize,
    pts: [Vec3; 3],
    norms: [Vec3; 3],
    mode: ShaderMode,
    time: f32
) {
    // proyecta a 2D pantalla
    let mut p = [(0.0, 0.0, 0.0); 3];
    for i in 0..3 {
        p[i] = (
            (pts[i].x * 0.5 + 0.5) * width as f32,
            (-pts[i].y * 0.5 + 0.5) * height as f32,
            pts[i].z
        );
    }

    // bounding box
    let minx = p.iter().map(|v| v.0).fold(f32::INFINITY, f32::min).floor().max(0.0) as i32;
    let maxx = p.iter().map(|v| v.0).fold(f32::NEG_INFINITY, f32::max).ceil().min(width as f32 - 1.0) as i32;
    let miny = p.iter().map(|v| v.1).fold(f32::INFINITY, f32::min).floor().max(0.0) as i32;
    let maxy = p.iter().map(|v| v.1).fold(f32::NEG_INFINITY, f32::max).ceil().min(height as f32 - 1.0) as i32;

    // área del triángulo
    let area = edge(p[0], p[1], p[2].0, p[2].1);
    if area.abs() < 1e-5 { return; }

    // píxel por píxel
    for y in miny..=maxy {
        for x in minx..=maxx {
            let w0 = edge(p[1], p[2], x as f32, y as f32);
            let w1 = edge(p[2], p[0], x as f32, y as f32);
            let w2 = edge(p[0], p[1], x as f32, y as f32);

            if (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0) {
                let w0n = w0 / area;
                let w1n = w1 / area;
                let w2n = w2 / area;
                let z = p[0].2 * w0n + p[1].2 * w1n + p[2].2 * w2n;
                let idx = (y as usize) * width + (x as usize);
                if z < zbuffer[idx] {
                    zbuffer[idx] = z;
                    let n = (norms[0] * w0n + norms[1] * w1n + norms[2] * w2n).normalize();
                    let view = Vec3::new(0.0, 0.0, 1.0);
                    let c = shade(mode, n, view, time);
                    buffer[idx] = rgb(c.x, c.y, c.z);
                }
            }
        }
    }
}

fn edge(a: (f32, f32, f32), b: (f32, f32, f32), x: f32, y: f32) -> f32 {
    (x - a.0) * (b.1 - a.1) - (y - a.1) * (b.0 - a.0)
}

fn rgb(r: f32, g: f32, b: f32) -> u32 {
    let r = (r.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (g.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (b.clamp(0.0, 1.0) * 255.0) as u32;
    (r << 16) | (g << 8) | b
}
