use std::f32::consts::PI;

use crate::transform::Transform;

const MAX_PITCH_DEGREES: f32 = 70.0;
const MIN_PITCH_DEGREES: f32 = -70.0;

const MAX_PITCH_RADIANS: f32 = MAX_PITCH_DEGREES * PI / 180.0;
const MIN_PITCH_RADIANS: f32 = MIN_PITCH_DEGREES * PI / 180.0;

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

    pub fn view(&self) -> glam::Mat4 {
        let mut view_matrix = glam::Mat4::from_quat(self.transform.rotation);
        let forward = self.transform.position + self.transform.forward;
        view_matrix.w_axis = glam::vec4(forward.x, forward.y, forward.z, 1.0);

        view_matrix.inverse()
    }

    pub fn projection(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(
            self.field_of_view * PI / 180.0,
            self.aspect_ratio,
            self.near_clipping_plane,
            self.far_clipping_plane,
        )
    }

    /// Rotates the Camera's yaw by the angle (in radians).
    /// If current yaw + angle is > 2PI then yaw will be set to 2PI - (yaw + angle)
    pub fn rotate_yaw(&mut self, angle: f32) {
        self.transform.yaw(angle)
    }

    /// Rotates the Camera's pitch by the angle (in radians) clamped to [[MIN_PITCH_RADIANS], [MAX_PITCH_RADIANS]].
    pub fn rotate_pitch(&mut self, angle: f32) {
        self.transform.pitch(angle)
    }
}
