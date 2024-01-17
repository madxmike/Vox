use std::ops::{Add, Sub};

use super::chunk::{CHUNK_BLOCK_DEPTH, CHUNK_BLOCK_HEIGHT, CHUNK_BLOCK_WIDTH};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPosition {
    #[inline]
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        BlockPosition { x, y, z }
    }

    pub fn to_chunk_origin(self) -> Self {
        let origin = BlockPosition::new(
            self.x - self.x.rem_euclid(CHUNK_BLOCK_WIDTH as i32),
            self.y - self.y.rem_euclid(CHUNK_BLOCK_HEIGHT as i32),
            self.z - self.z.rem_euclid(CHUNK_BLOCK_DEPTH as i32),
        );
        // dbg!(self);

        // dbg!(origin);
        origin
    }

    pub fn to_chunk_local_position(self) -> Self {
        BlockPosition::new(
            self.x.abs() % CHUNK_BLOCK_WIDTH as i32,
            self.y.abs() % CHUNK_BLOCK_HEIGHT as i32,
            self.z.abs() % CHUNK_BLOCK_DEPTH as i32,
        )
    }

    pub fn offset(&self, x: i32, y: i32, z: i32) -> Self {
        BlockPosition::new(self.x + x, self.y + y, self.z + z)
    }

    pub fn offset_x(&self, x: i32) -> Self {
        BlockPosition::new(self.x + x, self.y, self.z)
    }

    pub fn offset_y(&self, y: i32) -> Self {
        BlockPosition::new(self.x, self.y + y, self.z)
    }

    pub fn offset_z(&self, z: i32) -> Self {
        BlockPosition::new(self.x, self.y, self.z + z)
    }

    pub fn to_vec3(self) -> glam::Vec3 {
        glam::vec3(self.x as f32, self.y as f32, self.z as f32)
    }
}

impl Add for BlockPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        BlockPosition::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for BlockPosition {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        BlockPosition::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
