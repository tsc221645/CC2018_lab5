use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use std::mem;

// ========= Vertex =========
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal:   [f32; 3],
}

// 👉 Layout para wgpu 0.22
impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // position
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1, // normal
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// ========= Mesh =========
// (Si ya tienes tu Mesh y draw(), mantén los tuyos)
pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer:  wgpu::Buffer,
    pub index_count:   u32,
}

impl Mesh {
    pub fn from_vertices_indices(
        device: &wgpu::Device,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Self {
        let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh.vb"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh.ib"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self { vertex_buffer: vb, index_buffer: ib, index_count: indices.len() as u32 }
    }

    pub fn draw<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

// ========= Esfera UV de respaldo =========
// (Puedes borrar esto si cargas tu OBJ)
pub fn make_uv_sphere(
    device: &wgpu::Device,
    _queue: &wgpu::Queue,
    lat: u32,
    lon: u32,
) -> Mesh {
    let lat = lat.max(3);
    let lon = lon.max(3);
    let mut verts = Vec::<Vertex>::new();
    let mut idx   = Vec::<u32>::new();

    for y in 0..=lat {
        let v = y as f32 / lat as f32;
        let theta = v * std::f32::consts::PI;
        for x in 0..=lon {
            let u = x as f32 / lon as f32;
            let phi = u * std::f32::consts::TAU;
            let px = phi.sin() * theta.sin();
            let py = theta.cos();
            let pz = phi.cos() * theta.sin();
            let n = [px, py, pz];
            verts.push(Vertex { position: n, normal: n });
        }
    }
    let stride = lon + 1;
    for y in 0..lat {
        for x in 0..lon {
            let i0 = y * stride + x;
            let i1 = i0 + 1;
            let i2 = i0 + stride;
            let i3 = i2 + 1;
            idx.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }
    }

    Mesh::from_vertices_indices(device, &verts, &idx)
}
