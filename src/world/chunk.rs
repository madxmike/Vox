use crate::{
    cube_mesh::{self, CUBE_MESH},
    mesh::{Mesh, RuntimeMesh, StitchedMesh},
    transform::Position,
};

use super::{block::Block, block_position::BlockPosition};

pub const CHUNK_BLOCK_WIDTH: usize = 15;
pub const CHUNK_BLOCK_HEIGHT: usize = 15;
pub const CHUNK_BLOCK_DEPTH: usize = 15;
const CHUNK_SIZE: usize = CHUNK_BLOCK_WIDTH * CHUNK_BLOCK_HEIGHT * CHUNK_BLOCK_DEPTH;

pub enum ChunkAccessorError {
    PositionNotWithinChunk(BlockPosition),
}

pub struct Chunk {
    origin_position: BlockPosition,
    blocks: [Option<Block>; CHUNK_SIZE],
}

impl Chunk {
    pub fn new(origin_position: BlockPosition) -> Self {
        let mut chunk = Chunk {
            origin_position,
            blocks: [None; CHUNK_SIZE],
        };

        let mut skip = false;
        for i in 0..CHUNK_SIZE {
            if skip {
                skip = false;
                continue;
            }

            chunk.blocks[i] = Some(Block::default());
            skip = true;
        }
        chunk
    }
}

impl Chunk {
    // TODO (Michael): Move this type of logic to WorldRenderSystem
    pub fn build_chunk_mesh(&self) -> StitchedMesh {
        let mut stiched_mesh = StitchedMesh::default();

        for (i, block) in self.blocks.iter().enumerate() {
            if let None = block {
                continue;
            }
            let local_block_position = self.world_block_position(i);

            let mesh_verticies: Vec<[f32; 3]> = CUBE_MESH
                .verticies()
                .iter()
                .map(|vertex| {
                    [
                        vertex[0] + local_block_position.x as f32,
                        vertex[1] + local_block_position.y as f32,
                        vertex[2] + local_block_position.z as f32,
                    ]
                })
                .collect();

            let cube_mesh = RuntimeMesh {
                verticies: mesh_verticies,
                normals: CUBE_MESH.normals().to_vec(),
                indicies: CUBE_MESH.indicies().to_vec(),
            };

            stiched_mesh.stich(cube_mesh);
        }

        stiched_mesh
    }

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

        let chunk_local_position = world_position.to_chunk_local_position();

        let position_idx = (chunk_local_position.x
            + (chunk_local_position.y * CHUNK_BLOCK_WIDTH as i32)
            + (chunk_local_position.z * CHUNK_BLOCK_WIDTH as i32 * CHUNK_BLOCK_HEIGHT as i32))
            as usize;

        Ok(self.blocks[position_idx])
    }

    pub fn is_world_position_within(&self, world_position: BlockPosition) -> bool {
        self.origin_position == world_position.to_chunk_origin()
    }
}
