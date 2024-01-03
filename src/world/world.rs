use std::collections::HashMap;

use crate::transform::Position;

use super::{
    block::Block,
    block_position::BlockPosition,
    chunk::{self, Chunk, CHUNK_BLOCK_WIDTH},
};

pub struct World {
    // TODO (Michael): Later we want to move this to a chunk pool when we do loading / unloading
    pub chunks: HashMap<BlockPosition, Chunk>,
}

impl World {
    /// Get the block at the position from the loaded chunks.
    /// If a chunk the position is within is not loaded then this will return None.
    /// If the chunk is loaded, but the block is air, then this will return `None`.
    /// If the chunk is loaded, and the block is not air, then this will return `Some(block_at_position)`.
    pub fn get_block_at_position(&self, position: BlockPosition) -> Option<Block> {
        self.chunks
            .get(&position.to_chunk_origin())
            .and_then(|chunk| chunk.get_block_at_position(position).unwrap_or(None))
    }
}
