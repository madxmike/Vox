use glam::{vec3, Quat, Vec3};

pub type Position = glam::Vec3;
pub type Rotation = glam::Quat;

pub enum Axis {
    Forward,
    Right,
    Up,
}

#[derive(Clone, Copy, Default)]
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

    pub fn rotate(&mut self, axis: Axis, angle: f32) {
        self.rotation *= match axis {
            Axis::Forward => glam::Quat::from_axis_angle(self.forward, angle),
            Axis::Right => glam::Quat::from_axis_angle(self.right, angle),
            Axis::Up => glam::Quat::from_axis_angle(self.up, angle),
        };

        self.calculate_direction_vectors()
    }

    pub fn rotate_quat(&mut self, quat: Quat) {
        self.rotation *= quat
    }

    #[inline]
    pub fn roll(&mut self, angle: f32) {
        self.rotate(Axis::Forward, angle)
    }

    #[inline]
    pub fn pitch(&mut self, angle: f32) {
        self.rotate(Axis::Right, angle)
    }

    #[inline]
    pub fn yaw(&mut self, angle: f32) {
        self.rotate(Axis::Up, angle)
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.position.x += x;
        self.position.y += y;
        self.position.z += z;
    }

    pub fn translate_vec3(&mut self, vec: Vec3) {
        self.translate_along_axis(Axis::Forward, vec.z);
        self.translate_along_axis(Axis::Right, vec.x);
        self.translate_along_axis(Axis::Up, vec.y);
    }

    pub fn translate_along_axis(&mut self, axis: Axis, value: f32) {
        self.position += match axis {
            Axis::Forward => self.forward * value,
            Axis::Right => self.right * value,
            Axis::Up => self.up * value,
        };
    }

    fn calculate_direction_vectors(&mut self) {
        self.forward = Transform::forward(self.rotation);
        self.right = Transform::right(self.rotation);
        self.up = Transform::up(self.rotation);
    }

    #[inline]
    fn forward(rotation: glam::Quat) -> Vec3 {
        (rotation * vec3(0.0, 0.0, 1.0)).normalize()
    }

    #[inline]
    fn right(rotation: glam::Quat) -> Vec3 {
        (rotation * vec3(1.0, 0.0, 0.0)).normalize()
    }

    #[inline]
    fn up(rotation: glam::Quat) -> Vec3 {
        (rotation * vec3(0.0, -1.0, 0.0)).normalize()
    }
}
