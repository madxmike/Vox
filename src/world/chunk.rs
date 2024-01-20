use tokio::sync::RwLock;

use super::{block::Block, block_position::BlockPosition};

pub const CHUNK_BLOCK_WIDTH: usize = 16;
pub const CHUNK_BLOCK_HEIGHT: usize = 16;
pub const CHUNK_BLOCK_DEPTH: usize = 16;
const CHUNK_SIZE: usize = CHUNK_BLOCK_WIDTH * CHUNK_BLOCK_HEIGHT * CHUNK_BLOCK_DEPTH;

pub enum ChunkAccessorError {
    PositionNotWithinChunk(BlockPosition),
}

pub struct Chunk {
    origin_position: BlockPosition,
    pub blocks: [Block; CHUNK_SIZE],
}

impl Chunk {
    pub fn new(origin_position: BlockPosition) -> Self {
        Chunk {
            origin_position,
            blocks: [Block { id: 1, state: 0 }; CHUNK_SIZE],
        }
    }

    pub fn origin_position(&self) -> BlockPosition {
        self.origin_position
    }
}

impl Chunk {
    fn local_block_position(block_index: usize) -> BlockPosition {
        let local_z = block_index / (CHUNK_BLOCK_WIDTH * CHUNK_BLOCK_HEIGHT);
        let block_index = block_index - (local_z * CHUNK_BLOCK_WIDTH * CHUNK_BLOCK_HEIGHT);
        let local_y = block_index / CHUNK_BLOCK_WIDTH;
        let local_x = block_index % CHUNK_BLOCK_WIDTH;

        BlockPosition::new(local_x as i32, local_y as i32, local_z as i32)
    }

    #[inline]
    fn world_block_position(&self, block_index: usize) -> BlockPosition {
        Chunk::local_block_position(block_index) + self.origin_position
    }

    pub fn set_block_at_position(&mut self, world_position: BlockPosition, block: Block) {
        let chunk_local_position = world_position.to_chunk_local_position(); // 0, 0, 0

        let position_idx = (chunk_local_position.x
            + (chunk_local_position.y * CHUNK_BLOCK_WIDTH as i32)
            + (chunk_local_position.z * CHUNK_BLOCK_WIDTH as i32 * CHUNK_BLOCK_HEIGHT as i32))
            as usize;

        self.blocks[position_idx] = block;
    }

    /// Gets the block at the position, if one exists.
    /// If the position is not within this chunk, then this returns `None`
    /// If the block is air then this returns `None`.
    /// Otherwise, this returns `Some(block_at_position)`.
    pub fn get_block_at_position(
        &self,
        world_position: BlockPosition,
    ) -> Result<Option<Block>, ChunkAccessorError> {
        if !self.is_world_position_within(world_position) {
            return Err(ChunkAccessorError::PositionNotWithinChunk(world_position));
        }

        let chunk_local_position = world_position.to_chunk_local_position(); // 0, 0, 0

        let position_idx = (chunk_local_position.x
            + (chunk_local_position.y * CHUNK_BLOCK_WIDTH as i32)
            + (chunk_local_position.z * CHUNK_BLOCK_WIDTH as i32 * CHUNK_BLOCK_HEIGHT as i32))
            as usize;

        let block = self.blocks[position_idx];
        if block.is_air() {
            return Ok(None);
        }

        Ok(Some(block))
    }

    pub fn is_world_position_within(&self, world_position: BlockPosition) -> bool {
        self.origin_position == world_position.to_chunk_origin()
    }
}
