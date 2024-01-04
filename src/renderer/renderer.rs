use crate::{camera::Camera, mesh::Mesh};

pub trait Renderer {
    fn render(&mut self, camera: &Camera, mesh: Box<dyn Mesh>);
}
