use crate::math::*;

#[derive(Clone, Copy)]
pub enum ShaderMode {
    Star,
    Rock,
    Gas,
}

pub fn shade(mode: ShaderMode, n: Vec3, view: Vec3, time: f32) -> Vec3 {
    match mode {
        ShaderMode::Star => shade_star(n, view, time),
        ShaderMode::Rock => shade_rock(n, view, time),
        ShaderMode::Gas  => shade_gas(n, view, time),
    }
}

fn shade_star(n: Vec3, view: Vec3, time: f32) -> Vec3 {
    let lat = n.y * 0.5 + 0.5;
    let base_core = Vec3::new(1.0, 0.95, 0.6);
    let base_edge = Vec3::new(1.0, 0.4, 0.0);
    let mut c = mix(base_edge, base_core, lat);

    let turb = (n.x * 30.0 + time * 2.0).sin() * (n.y * 40.0 - time * 1.5).sin();
    let turb = turb * 0.5 + 0.5;
    let boil = Vec3::new(1.0, 0.8, 0.0);
    c = mix(c, boil, turb * 0.5);

    let spots = (n.x * 5.0 + (n.y * 7.0).sin() * 2.0).sin();
    let spots = smoothstep(0.5, 0.8, spots);
    c *= 1.0 - spots * 0.4;

    let rim = (1.0 - n.dot(&view).max(0.0)).powf(3.0);
    c + Vec3::new(1.0, 0.3, 0.0) * rim * 1.5
}

fn shade_rock(n: Vec3, view: Vec3, _time: f32) -> Vec3 {
    let cont = (n.x * 4.0 + (n.y * 4.0).sin() * 0.5 + 1.7 * n.z).sin() * 0.5 + 0.5;
    let desert = Vec3::new(0.6, 0.45, 0.3);
    let dark = Vec3::new(0.2, 0.15, 0.12);
    let mut c = mix(dark, desert, cont);

    let detail = (n.x * 30.0).sin() * (n.y * 30.0).sin() * (n.z * 30.0).sin();
    let detail = detail * 0.5 + 0.5;
    let lava = Vec3::new(1.0, 0.3, 0.05);
    let lava_mask = smoothstep(0.8, 1.0, detail);
    c = mix(c, lava, lava_mask);

    let light = Vec3::new(1.0, 1.0, 0.3).normalize();
    let diff = n.dot(&light).max(0.0);
    let lit = c * (0.2 + diff * 0.8);

    let rim = (1.0 - n.dot(&view).max(0.0)).powf(2.0);
    lit + Vec3::new(0.2, 0.4, 1.0) * rim * 0.3
}

fn shade_gas(n: Vec3, view: Vec3, time: f32) -> Vec3 {
    let lat = n.y * 0.5 + 0.5;
    let a = Vec3::new(0.9, 0.8, 0.6);
    let b = Vec3::new(0.8, 0.6, 0.4);
    let c1 = Vec3::new(0.7, 0.5, 0.3);
    let stripes = (lat * 50.0 + (n.x * 4.0 + time * 0.5).sin() * 2.0).sin() * 0.5 + 0.5;
    let mut c = mix(a, b, stripes);
    c = mix(c, c1, smoothstep(0.3, 0.7, lat));

    let flow = (n.x * 10.0 + time * 2.0).sin() * 0.5 + (n.z * 20.0 - time * 1.5).sin() * 0.5;
    c += Vec3::new(flow, flow, flow) * 0.05;

    let storm_center = Vec3::new(0.3, -0.1, 0.95).normalize();
    let storm = smoothstep(0.98, 1.0, n.dot(&storm_center));
    let storm_color = Vec3::new(1.0, 0.9, 0.7);
    c = mix(c, storm_color, storm);

    let rim = (1.0 - n.dot(&view).max(0.0)).powf(2.0);
    c + Vec3::new(1.0, 0.9, 0.8) * rim * 0.4
}
