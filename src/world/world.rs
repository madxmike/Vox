use std::collections::HashMap;



use super::{block::Block, block_position::BlockPosition, chunk::Chunk};

pub struct World {
    // TODO (Michael): Later we want to move this to a chunk pool when we do loading / unloading
    pub chunks: HashMap<BlockPosition, Chunk>,
}

impl World {
    pub fn set_block_at_position(&mut self, position: BlockPosition, block: Block) {
        self.chunks
            .get_mut(&position.to_chunk_origin())
            .and_then(|chunk| Some(chunk.set_block_at_position(position, block)));
    }

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
