use std::{borrow::Borrow, collections::HashMap};

use crate::world::{
    block_position::BlockPosition,
    chunk::{Chunk, CHUNK_BLOCK_DEPTH, CHUNK_BLOCK_HEIGHT, CHUNK_BLOCK_WIDTH},
};

use super::{block::Block, world::World};

pub struct WorldGenerationSettings {
    pub max_width: i32,
    pub max_height: i32,
    pub max_length: i32,
}

pub fn generate_world(_seed: u64, settings: WorldGenerationSettings) -> World {
    let mut chunks: HashMap<BlockPosition, Chunk> = HashMap::new();
    for x in -settings.max_width..settings.max_width {
        for y in 0..settings.max_height {
            for z in -settings.max_length..settings.max_length {
                let origin_position = BlockPosition::new(
                    x as i32 * CHUNK_BLOCK_WIDTH as i32,
                    y as i32 * CHUNK_BLOCK_HEIGHT as i32,
                    z as i32 * CHUNK_BLOCK_DEPTH as i32,
                );
                chunks.insert(origin_position, generate_chunk(origin_position));
            }
        }
    }

    World { chunks }
}

fn generate_chunk(origin_position: BlockPosition) -> Chunk {
    // For each block in the chunk
    // Apply generation rules to determine what block to place
    // TODO (Michael): We need a way to determine the rules
    // Probably something like generate biome from noise
    // The biome is a set of parameters that determine things like hilliness, water density, etc
    // Then do a decoration pass for things like trees, caves, etc
    // For now we can just add some basic noise

    Chunk::new(origin_position)
}
