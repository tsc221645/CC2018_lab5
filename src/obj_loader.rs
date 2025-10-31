use crate::math::*;
use std::fs;

#[derive(Clone)]
pub struct Vertex {
    pub pos: Vec3,
    pub normal: Vec3,
}

#[derive(Clone)]
pub struct Triangle {
    pub v0: Vertex,
    pub v1: Vertex,
    pub v2: Vertex,
}

pub fn load_obj(path: &str) -> Vec<Triangle> {
    let data = fs::read_to_string(path).expect("No se pudo leer el .obj");
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut triangles = Vec::new();

    for line in data.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { continue; }

        match parts[0] {
            "v" => {
                let x: f32 = parts[1].parse().unwrap();
                let y: f32 = parts[2].parse().unwrap();
                let z: f32 = parts[3].parse().unwrap();
                positions.push(Vec3::new(x, y, z));
            }
            "vn" => {
                let x: f32 = parts[1].parse().unwrap();
                let y: f32 = parts[2].parse().unwrap();
                let z: f32 = parts[3].parse().unwrap();
                normals.push(Vec3::new(x, y, z));
            }
            "f" => {
                let mut verts = vec![];
                for p in &parts[1..] {
                    let parts: Vec<&str> = p.split('/').collect();
                    let vi = parts[0].parse::<usize>().unwrap() - 1;
                    let ni = if parts.len() >= 3 && !parts[2].is_empty() {
                        parts[2].parse::<usize>().unwrap() - 1
                    } else {
                        vi
                    };
                    verts.push(Vertex {
                        pos: positions[vi],
                        normal: if normals.is_empty() { positions[vi].normalize() } else { normals[ni] },
                    });
                }
                if verts.len() == 3 {
                    triangles.push(Triangle {
                        v0: verts[0].clone(),
                        v1: verts[1].clone(),
                        v2: verts[2].clone(),
                    });
                }
            }
            _ => {}
        }
    }
    triangles
}
