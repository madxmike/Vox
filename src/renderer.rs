use crate::camera::Camera;

pub trait Renderer {
    fn render(&mut self, camera: &Camera);
}
