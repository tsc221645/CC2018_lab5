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

// Shaders de planetas (equivalente a tu código Rust)
fn star_shader(normal: vec3<f32>, time: f32) -> vec3<f32> {
    let noise = fract(sin(dot(normal, vec3<f32>(12.9898, 78.233, 45.164))) * 43758.5453);
    let flare = pow(max(0.0, sin(time * 2.0 + noise * 10.0)), 3.0);
    return vec3<f32>(1.0, 0.8, 0.2) + vec3<f32>(0.5, 0.2, 0.0) * flare;
}

fn rock_shader(normal: vec3<f32>, pos: vec3<f32>, time: f32) -> vec3<f32> {
    let noise = fract(sin(dot(pos * 10.0, vec3<f32>(12.9898, 78.233, 45.164))) * 43758.5453);
    return vec3<f32>(0.4, 0.3, 0.25) * (0.7 + noise * 0.3);
}

fn gas_shader(normal: vec3<f32>, pos: vec3<f32>, time: f32) -> vec3<f32> {
    let band = sin(pos.y * 8.0 + time) * 0.5 + 0.5;
    let base = mix(vec3<f32>(0.8, 0.6, 0.3), vec3<f32>(0.9, 0.7, 0.4), band);
    return base;
}

fn earth_shader(normal: vec3<f32>, pos: vec3<f32>, time: f32) -> vec3<f32> {
    let noise = fract(sin(dot(pos * 5.0, vec3<f32>(12.9898, 78.233, 45.164))) * 43758.5453);
    let ocean = vec3<f32>(0.1, 0.3, 0.6);
    let land = vec3<f32>(0.2, 0.5, 0.2);
    return mix(ocean, land, step(0.4, noise));
}

fn moon_shader(normal: vec3<f32>, pos: vec3<f32>, time: f32) -> vec3<f32> {
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
        case 1u: { color = rock_shader(normal, input.world_pos, uniforms.time); }
        case 2u: { color = gas_shader(normal, input.world_pos, uniforms.time); }
        case 3u: { color = earth_shader(normal, input.world_pos, uniforms.time); }
        case 4u: { color = moon_shader(normal, input.world_pos, uniforms.time); }
        default: { color = vec3<f32>(1.0, 0.0, 1.0); }
    }
    
    color = color * (0.3 + diffuse * 0.7);
    
    return vec4<f32>(color, 1.0);
}