use glam::{vec3, Mat4, Vec3};

pub type Position = glam::Vec3;
pub type Rotation = glam::Quat;

pub struct Transform {
    pub position: Position,
    pub rotation: Rotation,

    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl Transform {
    pub fn new(position: Vec3, rotation_euler: Vec3) -> Self {
        let rotation = glam::Quat::from_euler(
            glam::EulerRot::XYZ,
            rotation_euler.x,
            rotation_euler.y,
            rotation_euler.z,
        );
        let forward = Transform::forward(rotation);
        let right = Transform::right(rotation);
        let up = Transform::up(rotation);

        Transform {
            position,
            rotation,
            forward,
            right,
            up,
        }
    }

    #[inline]
    pub fn rotation_eurler(&self) -> (f32, f32, f32) {
        self.rotation.to_euler(glam::EulerRot::XYZ)
    }

    pub fn rotate_yaw(&mut self, mut angle: f32) {
        self.rotation *= glam::Quat::from_axis_angle(self.up, angle);
        self.calculate_direction_vectors()
    }

    fn calculate_direction_vectors(&mut self) {
        self.forward = Transform::forward(self.rotation);
        self.right = Transform::right(self.rotation);
        self.up = Transform::up(self.rotation);
    }

    #[inline]
    fn forward(rotation: glam::Quat) -> Vec3 {
        rotation.conjugate() * vec3(0.0, 0.0, -1.0)
    }

    #[inline]
    fn right(rotation: glam::Quat) -> Vec3 {
        rotation.conjugate() * vec3(1.0, 0.0, 0.0)
    }

    #[inline]
    fn up(rotation: glam::Quat) -> Vec3 {
        rotation.conjugate() * vec3(0.0, -1.0, 0.0)
    }
}
