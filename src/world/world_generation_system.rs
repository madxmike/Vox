use std::collections::HashMap;

use crate::world::{
    block_position::BlockPosition,
    chunk::{Chunk, CHUNK_BLOCK_WIDTH},
};

use super::world::World;

pub fn generate_world(seed: u64) -> World {
    const NUM_CHUNKS: u32 = 1;

    let mut chunks: HashMap<BlockPosition, Chunk> = HashMap::new();
    for i in 0..NUM_CHUNKS {
        let origin_position = BlockPosition::new(i as i32 * CHUNK_BLOCK_WIDTH as i32, 0, 0);
        chunks.insert(origin_position, Chunk::new(origin_position));
    }

    World { chunks }
}
