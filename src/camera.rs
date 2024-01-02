use std::f32::consts::PI;

use glam::vec3;

use crate::transform::Transform;

// TODO (Michael): Could we apply some nicer types here to ensure correctness?
pub struct Camera {
    pub transform: Transform,
    pub near_clipping_plane: f32,
    pub far_clipping_plane: f32,
    pub field_of_view: f32,
    pub aspect_ratio: f32,
}

impl Camera {
    // fn new() -> Self {}
    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    pub fn mvp(&self) -> (glam::Mat4, glam::Mat4, glam::Mat4) {
        let model = glam::Mat4::from_quat(self.transform.rotation);
        let view = glam::Mat4::look_at_rh(
            self.transform.position,
            // TODO (Michael): Create vectors for this, possibly off the transform?
            self.transform.position + vec3(0.0, 0.0, -1.0),
            glam::vec3(0.0, -1.0, 0.0),
        );
        let projection = glam::Mat4::perspective_rh(
            self.field_of_view * PI / 180.0,
            self.aspect_ratio,
            self.near_clipping_plane,
            self.far_clipping_plane,
        );

        (model, view, projection)
    }
}
