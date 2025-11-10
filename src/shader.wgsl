// Modes: 1=Star, 2=Rock, 3=Gas, 4=Earth, 5=Moon

struct Globals {
    time: f32,
    zoom: f32,
    mode: u32,
    _pad: u32,
};
@group(0) @binding(0)
var<uniform> globals: Globals;

struct VSIn {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
};

struct VSOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0)       normal:   vec3<f32>,
};

@vertex
fn vs_main(in: VSIn) -> VSOut {
    var out: VSOut;
    out.clip_pos = vec4<f32>(in.position * globals.zoom, 1.0);
    out.normal   = normalize(in.normal);
    return out;
}

fn saturate3(x: vec3<f32>) -> vec3<f32> {
    return clamp(x, vec3<f32>(0.0), vec3<f32>(1.0));
}

// --- simple trig "noise" ---
fn noise3(p: vec3<f32>, t: f32) -> f32 {
    return sin(p.x * 1.7 + sin(p.z * 0.9) + t * 0.05) *
           cos(p.y * 1.3 - cos(p.x * 0.6) + t * 0.04) *
           sin(p.z * 1.9 + sin(p.y * 0.7) + t * 0.03);
}

fn fbm(p0: vec3<f32>, t: f32) -> f32 {
    var p = p0;
    var a = 1.0;
    var f = 1.5;
    var acc = 0.0;
    for (var i = 0; i < 5; i = i + 1) {
        acc = acc + abs(noise3(p * f, t)) * a;
        f = f * 2.1;
        a = a * 0.5;
    }
    return clamp(acc, 0.0, 1.0);
}
// Coordenadas esféricas estables a partir de la normal
fn spherical_uv(n: vec3<f32>) -> vec2<f32> {
    let u = atan2(n.z, n.x) / (2.0 * 3.14159265) + 0.5;
    let v = n.y * 0.5 + 0.5;
    return vec2<f32>(u, v);
}


// ---- Star ----
fn shade_star(n: vec3<f32>, v: vec3<f32>, t: f32) -> vec3<f32> {
    // Base: mezcla de azules y blancos
    var c = mix(vec3<f32>(0.2, 0.4, 1.0), vec3<f32>(0.9, 0.95, 1.0), n.y * 0.5 + 0.5);

    // Turbulencia tipo plasma solar
    let plasma = abs(sin(n.x * 30.0 + t * 3.0) * cos(n.y * 25.0 - t * 2.0) * sin(n.z * 20.0 + t));
    c = mix(c, vec3<f32>(0.8, 0.9, 1.0), plasma * 0.6);

    // Filamentos más brillantes y manchas
    let filaments = abs(sin(n.x * 10.0) * cos(n.y * 15.0) * sin(n.z * 12.0));
    c = mix(c, vec3<f32>(1.0, 0.8, 0.6), smoothstep(0.7, 0.95, filaments) * 0.5);

    // Brillo atmosférico
    let rim = pow(1.0 - max(dot(n, v), 0.0), 2.0);
    return c + vec3<f32>(0.6, 0.8, 1.0) * rim * 1.5;
}

// ---- Rock (with heart) ----
fn shade_rock(n: vec3<f32>, v: vec3<f32>, t: f32) -> vec3<f32> {
    let p = n * 4.0;
    var relief = fbm(p, t * 0.2);
    let cracks = abs(sin(n.x * 80.0 + t * 0.5) * cos(n.z * 60.0 - t * 0.4));
    relief = clamp(relief + cracks * 0.2, 0.0, 1.0);

    // Tonos terrosos ocres-grises
    var c = mix(vec3<f32>(0.15, 0.1, 0.08), vec3<f32>(0.6, 0.5, 0.4), relief);
    c = mix(c, vec3<f32>(0.8, 0.7, 0.55), smoothstep(0.6, 0.9, relief));

    // Sombras tipo cráter
    let crater = smoothstep(0.8, 1.0, abs(sin(n.x * 20.0) * cos(n.z * 25.0)));
    c = mix(c, vec3<f32>(0.05, 0.05, 0.05), crater * 0.7);

    let light_dir = normalize(vec3<f32>(0.7, 0.5, 0.6));
    let diff = max(dot(n, light_dir), 0.0);
    let h = normalize(light_dir + v);
    let spec = pow(max(dot(n, h), 0.0), 20.0) * 0.3;

    return c * (0.4 + diff) + spec;
}

// ---- Gas ----
fn shade_gas(n: vec3<f32>, v: vec3<f32>, t: f32) -> vec3<f32> {
    // Bandas fluidas animadas
    let lat = n.y * 3.14;
    var bands = sin(lat * 8.0 + t * 0.6) * 0.5 + 0.5;
    let swirl = sin(n.x * 10.0 + sin(n.z * 6.0) + t * 0.4);
    bands = mix(bands, swirl, 0.3);

    var c = mix(vec3<f32>(0.3, 0.1, 0.3), vec3<f32>(0.8, 0.4, 0.7), bands);
    c = mix(c, vec3<f32>(0.95, 0.85, 0.95), smoothstep(0.7, 1.0, bands));

    // Nebulosidad sutil
    let haze = abs(sin(n.z * 15.0 + t * 0.5) * cos(n.y * 10.0 - t * 0.3));
    c += vec3<f32>(0.1, 0.08, 0.12) * haze * 0.5;

    // Sombra atmosférica
    let rim = pow(1.0 - max(dot(n, v), 0.0), 2.5);
    return c + vec3<f32>(0.5, 0.4, 0.6) * rim * 0.3;
}


// ---- Earth ----
fn shade_earth(n: vec3<f32>, v: vec3<f32>, t: f32) -> vec3<f32> {
    // Continentes y océanos
    let land = fbm(n * 3.0, t * 0.1);
    let ocean_mask = smoothstep(0.4, 0.55, land);
    var surface = mix(vec3<f32>(0.0, 0.15, 0.45), vec3<f32>(0.15, 0.4, 0.15), ocean_mask);

    // Nubes con ruido animado y bordes suaves
    let cloud_noise = fbm(n * 6.0 + vec3<f32>(t * 0.1, t * 0.1, 0.0), t);
    let clouds = smoothstep(0.55, 0.7, cloud_noise);
    surface = mix(surface, vec3<f32>(1.0, 1.0, 1.0), clouds * 0.4);

    // Iluminación y brillo
    let light_dir = normalize(vec3<f32>(0.6, 0.4, 0.7));
    let diff = max(dot(n, light_dir), 0.0);
    let h = normalize(light_dir + v);
    let spec = pow(max(dot(n, h), 0.0), 50.0) * 0.25;

    return surface * (0.25 + diff * 0.9) + spec;
}


// ---- Moon ----
fn shade_moon(n: vec3<f32>, v: vec3<f32>, t: f32) -> vec3<f32> {
    // Coordenadas suavizadas para evitar distorsión
    let uv = spherical_uv(n);
    let p = vec3<f32>(uv.x * 6.2831, uv.y * 3.1415, 0.0);


    // Ruido fractal para relieve
    var base_noise = fbm(p, t * 0.1);
    var detail = fbm(p * 4.0 + vec3<f32>(0.0, t * 0.2, 0.0), t * 0.1);
    var terrain = clamp(base_noise * 0.7 + detail * 0.3, 0.0, 1.0);

    // Colores tipo Marte: ocres, naranjas, marrones
    let dark_rock = vec3<f32>(0.25, 0.1, 0.05);
    let mid_rock  = vec3<f32>(0.55, 0.25, 0.1);
    let light_rock = vec3<f32>(0.85, 0.55, 0.3);
    var c = mix(dark_rock, mid_rock, terrain);
    c = mix(c, light_rock, smoothstep(0.6, 0.9, terrain));

    // Cráteres suaves (interferencias esféricas)
    let crater_pattern = abs(sin(p.x * 6.0) * cos(p.z * 6.0) * sin(p.y * 6.0));
    let craters = smoothstep(0.75, 0.95, crater_pattern);
    c = mix(c, dark_rock * 0.6, craters * 0.8);

    // Luz direccional tipo sol
    let light_dir = normalize(vec3<f32>(0.6, 0.4, 0.7));
    let diff = max(dot(n, light_dir), 0.0);
    let h = normalize(light_dir + v);
    let spec = pow(max(dot(n, h), 0.0), 32.0);

    // Oclusión ambiental simple
    let ao = 0.6 + 0.4 * (1.0 - terrain);

    // Rim light atmosférico (brillo tenue al borde)
    let rim = pow(1.0 - max(dot(n, v), 0.0), 2.5);

    // Combinación final
    var color = c * (ao * 0.3 + diff * 0.9);
    color += vec3<f32>(1.0, 0.8, 0.6) * spec * 0.2;
    color += vec3<f32>(1.0, 0.5, 0.3) * rim * 0.25;

    return saturate3(color);
}

// ---- Iridescent / Crystal ----
fn shade_iridescent(n: vec3<f32>, v: vec3<f32>, t: f32) -> vec3<f32> {
    // Interferencia óptica simulada: mezcla de colores espectrales según ángulo de visión
    let angle = pow(1.0 - max(dot(n, v), 0.0), 1.5);
    let hue_shift = sin(t * 0.3 + n.x * 3.0 + n.y * 5.0 + n.z * 7.0) * 0.5 + 0.5;
    let base_hue = angle * 0.6 + hue_shift * 0.4;

    // Convertir de HSV a RGB manualmente
    let c = 1.0;
    let x = 1.0 - abs((base_hue * 6.0) % 2.0 - 1.0);
    var rgb = vec3<f32>(0.0);
    if (base_hue < 1.0/6.0)      { rgb = vec3<f32>(c, x, 0.0); }
    else if (base_hue < 2.0/6.0) { rgb = vec3<f32>(x, c, 0.0); }
    else if (base_hue < 3.0/6.0) { rgb = vec3<f32>(0.0, c, x); }
    else if (base_hue < 4.0/6.0) { rgb = vec3<f32>(0.0, x, c); }
    else if (base_hue < 5.0/6.0) { rgb = vec3<f32>(x, 0.0, c); }
    else                         { rgb = vec3<f32>(c, 0.0, x); }

    // Detalles cristalinos: ruido facetado animado
    let pattern = abs(sin(n.x * 25.0 + t * 0.8) * cos(n.y * 30.0 - t * 0.6));
    let crystal = smoothstep(0.6, 0.9, pattern);
    rgb = mix(rgb, vec3<f32>(0.9, 0.9, 0.95), crystal * 0.4);

    // Iluminación especular intensa tipo metal
    let light_dir = normalize(vec3<f32>(0.6, 0.4, 0.7));
    let diff = max(dot(n, light_dir), 0.0);
    let h = normalize(light_dir + v);
    let spec = pow(max(dot(n, h), 0.0), 100.0);

    // Reflejo tipo “clear coat”
    let coat = pow(1.0 - max(dot(n, v), 0.0), 3.5);
    let reflection = mix(rgb, vec3<f32>(1.0, 1.0, 1.0), coat * 0.6);

    return saturate3(reflection * (0.3 + diff * 0.8) + spec * 0.8);
}


@fragment
fn fs_main(@location(0) n_in: vec3<f32>) -> @location(0) vec4<f32> {
    var n = normalize(n_in);
    let v = normalize(vec3<f32>(0.0, 0.0, 1.0));
    let t = globals.time;
    let mode = globals.mode;

    // Luz direccional simulando un sol
    let light_dir = normalize(vec3<f32>(0.6, 0.4, 0.7));
    let diff = max(dot(n, light_dir), 0.0);
    let rim = pow(1.0 - max(dot(n, v), 0.0), 3.0);

    // Perturba la normal con ruido procedural
    let bump = fbm(n * 3.0, t * 0.2);
    n = normalize(n + (bump - 0.5) * 0.3);

    // Selección de color base según modo
    var color = vec3<f32>(0.0);
    if (mode == 1u)      { color = shade_star(n, v, t); }
    else if (mode == 2u) { color = shade_rock(n, v, t); }
    else if (mode == 3u) { color = shade_gas(n, v, t); }
    else if (mode == 4u) { color = shade_earth(n, v, t); }
    else if (mode == 5u) { color = shade_moon(n, v, t); }
    else if (mode == 6u) { color = shade_iridescent(n, v, t); }

    // Aplicar luz difusa + rimlight para volumen
    var lit = color * (0.15 + diff * 0.9);
    lit += rim * 0.25 * vec3<f32>(1.0, 1.0, 1.0);

    return vec4<f32>(saturate3(lit), 1.0);
}