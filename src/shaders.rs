use crate::math::*;

// --------------------------------------------------
// Tipos de shader
// --------------------------------------------------
#[derive(Clone, Copy)]
pub enum ShaderMode {
    Star,
    Rock,
    Gas,
    Earth, 
    Moon,
}

// --------------------------------------------------
// Selector de shader
// --------------------------------------------------
pub fn shade(mode: ShaderMode, n: Vec3, view: Vec3, time: f32) -> Vec3 {
    match mode {
        ShaderMode::Star  => shade_star(n, view, time),
        ShaderMode::Rock  => shade_rock(n, view, time),
        ShaderMode::Gas   => shade_gas(n, view, time),
        ShaderMode::Earth => shade_earth(n, view, time),
        ShaderMode::Moon => shade_moon(n, view, time),

    }
}

// --------------------------------------------------
// ☀️ Estrella: plasma solar turbulento y animado
// --------------------------------------------------
fn shade_star(n: Vec3, view: Vec3, time: f32) -> Vec3 {
    let lat = n.y * 0.5 + 0.5;
    let core = Vec3::new(1.0, 0.95, 0.6);
    let edge = Vec3::new(1.0, 0.5, 0.0);
    let mut c = mix(edge, core, lat);

    // turbulencia dinámica (ondas internas)
    let plasma = ((n.x * 25.0 + time * 3.0).sin()
        * (n.y * 35.0 - time * 2.0).cos()
        * (n.z * 20.0 + time).sin())
        .abs();
    c = mix(c, Vec3::new(1.0, 0.8, 0.1), plasma * 0.6);

    // manchas solares
    let spots = ((n.x * 8.0).sin() * (n.y * 8.0).cos() * (n.z * 8.0).sin()).abs();
    let mask = smoothstep(0.6, 0.9, spots);
    c = mix(c, Vec3::new(0.3, 0.1, 0.05), mask * 0.4);

    // brillo de borde (efecto corona)
    let rim = (1.0 - n.dot(&view).max(0.0)).powf(3.0);
    c + Vec3::new(1.0, 0.3, 0.0) * rim * 1.2
}

// --------------------------------------------------
// 🪨 Planeta rocoso: relieve, vetas y lava
// --------------------------------------------------
fn shade_rock(n: Vec3, view: Vec3, time: f32) -> Vec3 {
    let relief = ((n.x * 10.0 + n.z * 7.0).sin()
        * (n.y * 10.0 + time * 0.5).cos())
        .abs();

    let sand = Vec3::new(0.8, 0.65, 0.45);
    let dirt = Vec3::new(0.3, 0.22, 0.1);
    let rock = Vec3::new(0.4, 0.3, 0.2);
    let mut c = mix(dirt, sand, relief);

    // vetas minerales
    let veins = ((n.x * 20.0).sin() * (n.y * 25.0).cos() * (n.z * 30.0).sin()).abs();
    let veins_mask = smoothstep(0.7, 0.9, veins);
    c = mix(c, rock, veins_mask * 0.5);

    // lava brillante
    let lava_noise = ((n.x * 40.0).sin()
        * (n.y * 40.0).sin()
        * (n.z * 40.0).cos())
        .abs();
    if lava_noise > 0.95 {
        c = mix(c, Vec3::new(1.0, 0.3, 0.05), (lava_noise - 0.95) * 10.0);
    }

    // iluminación direccional + rim light
    let light = Vec3::new(0.8, 1.0, 0.5).normalize();
    let diff = n.dot(&light).max(0.0);
    let base = c * (0.25 + diff * 0.9);

    let rim = (1.0 - n.dot(&view).max(0.0)).powf(2.0);
    base + Vec3::new(0.1, 0.3, 0.6) * rim * 0.4
}

// --------------------------------------------------
// ☁️ Gigante gaseoso: bandas y tormentas animadas
// --------------------------------------------------
fn shade_gas(n: Vec3, view: Vec3, time: f32) -> Vec3 {
    // bandas principales
    let lat = n.y * 0.5 + 0.5;
    let stripe_pattern =
        (lat * 20.0 + (n.x * 2.0 + time * 0.5).sin() * 3.0).sin().abs();
    let upper = Vec3::new(0.9, 0.75, 0.55);
    let lower = Vec3::new(0.6, 0.45, 0.35);
    let mut c = mix(lower, upper, stripe_pattern);

    // nubes y flujo atmosférico
    let turbulence =
        ((n.x * 10.0 + time).sin() * (n.z * 15.0 - time * 0.8).cos()).abs();
    c = mix(c, Vec3::new(0.8, 0.7, 0.6), turbulence * 0.4);

    // tormenta localizada
    let storm_center = Vec3::new(0.4, -0.2, 0.9).normalize();
    let s_intensity = smoothstep(0.98, 1.0, n.dot(&storm_center));
    let storm_color = Vec3::new(1.0, 0.9, 0.7);
    c = mix(c, storm_color, s_intensity);

    // gradiente polar y luz de borde
    let poles = smoothstep(0.6, 1.0, n.y.abs());
    c = mix(c, Vec3::new(0.3, 0.4, 0.6), poles * 0.3);

    let rim = (1.0 - n.dot(&view).max(0.0)).powf(2.0);
    c + Vec3::new(0.4, 0.4, 0.5) * rim * 0.4
}

fn shade_moon(n: Vec3, view: Vec3, time: f32) -> Vec3 {
    // ---------------------------
    // 1️⃣ Base rocosa gris
    // ---------------------------
    let base = (n.x * 4.0 + n.z * 2.5).sin() * (n.y * 3.0).cos();
    let detail = ((n.x * 15.0).sin() * (n.z * 10.0).cos()).abs();
    let rough = smoothstep(0.2, 0.8, detail);
    let mut surface = mix(
        Vec3::new(0.25, 0.25, 0.25),
        Vec3::new(0.5, 0.5, 0.5),
        rough,
    );

    // ---------------------------
    // 2️⃣ Cráteres (hundimientos oscuros)
    // ---------------------------
    let crater_pattern = (
        (n.x * 30.0 + (n.z * 40.0).sin()).sin() *
        (n.y * 35.0 - time * 0.05).cos()
    ).abs();

    let craters = smoothstep(0.6, 0.9, crater_pattern);
    surface = mix(surface, Vec3::new(0.15, 0.15, 0.15), craters * 0.8);

    // ---------------------------
    // 3️⃣ Luz direccional + relieve suave
    // ---------------------------
    let light_dir = Vec3::new(0.6, 0.4, 0.7).normalize();
    let diff = n.dot(&light_dir).max(0.0);
    let ambient = 0.25;
    let mut color = surface * (ambient + diff * 0.9);

    // ---------------------------
    // 4️⃣ Halo sutil (luz reflejada)
    // ---------------------------
    let rim = (1.0 - n.dot(&view).max(0.0)).powf(3.0);
    color += Vec3::new(0.7, 0.7, 0.8) * rim * 0.2;

    // ---------------------------
    // 5️⃣ Clamp manual
    // ---------------------------
    Vec3::new(
        color.x.clamp(0.0, 1.0),
        color.y.clamp(0.0, 1.0),
        color.z.clamp(0.0, 1.0),
    )
}


// --------------------------------------------------
// 🌍 Tierra con continentes y nubes
// --------------------------------------------------
fn shade_earth(n: Vec3, view: Vec3, time: f32) -> Vec3 {
    // ---------------------------
    // 1️⃣ Continentes y océanos (ruido suave)
    // ---------------------------
    let base = (n.x * 3.7 + (n.y * 2.1).sin() + n.z * 1.3).sin();
    let detail = ((n.x * 7.5 + n.z * 5.3).cos() * (n.y * 4.7).sin()).sin();
    let mask = smoothstep(-0.2, 0.4, base + detail * 0.5);

    let ocean = Vec3::new(0.0, 0.15, 0.45);
    let shore = Vec3::new(0.0, 0.3, 0.25);
    let land = Vec3::new(0.15, 0.35, 0.12);
    let desert = Vec3::new(0.55, 0.45, 0.2);

    let land_mix = mix(land, desert, smoothstep(0.4, 1.0, n.y.abs()));
    let mut surface = mix(ocean, mix(shore, land_mix, mask), mask);

    // polos blancos
    let poles = smoothstep(0.65, 0.9, n.y.abs());
    surface = mix(surface, Vec3::new(0.8, 0.9, 1.0), poles * 0.7);

    // ---------------------------
    // 2️⃣ Iluminación
    // ---------------------------
    let light_dir = Vec3::new(0.6, 0.4, 0.7).normalize();
    let diff = n.dot(&light_dir).max(0.0);
    let ambient = 0.25;
    let mut color = surface * (ambient + diff * 0.9);

    // ---------------------------
    // 3️⃣ Nubes suaves animadas
    // ---------------------------
    let cloud_pattern = (
        (n.x * 8.0 + time * 0.3).sin()
        + (n.y * 6.0 - time * 0.4).cos()
        + (n.z * 9.0 + time * 0.2).sin()
    ) / 3.0;

    let clouds = smoothstep(0.55, 0.75, cloud_pattern); // más difusas
    let cloud_color = Vec3::new(1.0, 1.0, 1.0);
    color = mix(color, cloud_color, clouds * 0.35);


    // clamp manual
    Vec3::new(
        color.x.clamp(0.0, 1.0),
        color.y.clamp(0.0, 1.0),
        color.z.clamp(0.0, 1.0),
    )
}


