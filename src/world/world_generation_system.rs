use std::collections::HashMap;

use crate::world::{
    block_position::BlockPosition,
    chunk::{Chunk, CHUNK_BLOCK_DEPTH, CHUNK_BLOCK_HEIGHT, CHUNK_BLOCK_WIDTH},
};

use super::world::World;

pub fn generate_world(_seed: u64) -> World {
    const NUM_CHUNKS: u32 = 10;

    let mut chunks: HashMap<BlockPosition, Chunk> = HashMap::new();
    for x in 0..12 {
        for y in 0..10 {
            for z in 0..10 {
                let origin_position = BlockPosition::new(
                    x as i32 * CHUNK_BLOCK_WIDTH as i32,
                    y as i32 * CHUNK_BLOCK_HEIGHT as i32,
                    z as i32 * CHUNK_BLOCK_DEPTH as i32,
                );
                chunks.insert(origin_position, Chunk::new(origin_position));
            }
        }
    }

    World { chunks }
}
