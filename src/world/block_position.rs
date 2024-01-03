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
        BlockPosition::new(
            self.x - (self.x % CHUNK_BLOCK_WIDTH as i32),
            self.y - (self.y % CHUNK_BLOCK_HEIGHT as i32),
            self.z - (self.z % CHUNK_BLOCK_DEPTH as i32),
        )
    }

    pub fn to_chunk_local_position(self) -> Self {
        let chunk_origin = self.clone().to_chunk_origin();

        BlockPosition::new(
            self.x % chunk_origin.x.abs(),
            self.y % chunk_origin.y,
            self.z % chunk_origin.z.abs(),
        )
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
