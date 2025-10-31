mod math;
mod obj_loader;
mod shaders;
mod renderer;

use obj_loader::load_obj;
use renderer::render;

fn main() {
    let triangles = load_obj("esfera.obj");
    println!("Triángulos cargados: {}", triangles.len());
    render(&triangles);
}
