use std::collections::HashMap;



use super::{
    block::Block,
    block_position::BlockPosition,
    chunk::{Chunk},
    direction::Direction,
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

    /// Gets the 6 neighbor blocks of a block at the position.
    /// If a block is air, the neighbor block will be [None].
    /// This will always be in [Direction] order.
    pub fn get_neighbors(&self, position: BlockPosition) -> Vec<(Direction, Option<Block>)> {
        vec![
            (
                Direction::North,
                self.get_block_at_position(position.offset(0, 0, 1)),
            ),
            (
                Direction::South,
                self.get_block_at_position(position.offset(0, 0, -1)),
            ),
            (
                Direction::East,
                self.get_block_at_position(position.offset(-1, 0, 0)),
            ),
            (
                Direction::West,
                self.get_block_at_position(position.offset(1, 0, 0)),
            ),
            (
                Direction::Up,
                self.get_block_at_position(position.offset(0, 1, 0)),
            ),
            (
                Direction::Down,
                self.get_block_at_position(position.offset(0, -1, 0)),
            ),
        ]
    }
}
