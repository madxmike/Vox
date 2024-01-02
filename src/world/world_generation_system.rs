use crate::world::chunk::Chunk;

use super::world::World;

pub fn generate_world(seed: u64) -> World {
    const NUM_CHUNKS: u32 = 10;

    let mut chunks: Vec<Chunk> = vec![];
    for i in 0..NUM_CHUNKS {
        chunks.push(Chunk::default())
    }

    World { chunks }
}
