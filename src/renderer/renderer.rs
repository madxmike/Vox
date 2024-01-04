use crate::{camera::Camera, mesh::Mesh};

pub trait Renderer {
    fn default_lit(&mut self, camera: &Camera, mesh: Box<dyn Mesh>);
}
