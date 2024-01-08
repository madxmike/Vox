use glam::{Mat4, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct MVP {
    pub model: Vec3,
    pub view: Mat4,
    pub projection: Mat4,
}

impl MVP {
    pub fn to_clip_space(&self) -> Mat4 {
        self.projection * self.view * Mat4::from_translation(self.model)
    }
}
