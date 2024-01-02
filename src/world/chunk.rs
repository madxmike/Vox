use crate::{
    cube_mesh::CUBE_MESH,
    mesh::{Mesh, RuntimeMesh},
    transform::Position,
};

use super::block::Block;

const BLOCKS_WIDTH: usize = 16;
const BLOCKS_HEIGHT: usize = 16;
const BLOCKS_DEPTH: usize = 16;
const CHUNK_SIZE: usize = BLOCKS_WIDTH * BLOCKS_HEIGHT * BLOCKS_DEPTH;

pub struct Chunk {
    origin_position: Position,
    blocks: [Block; CHUNK_SIZE],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            origin_position: glam::vec3(0.0, 0.0, 0.0),
            blocks: [Default::default(); CHUNK_SIZE],
        }
    }
}

impl Chunk {
    pub fn build_chunk_mesh(&self) -> RuntimeMesh {
        let mut mesh_verticies: Vec<[f32; 3]> = vec![];
        let mut mesh_normals: Vec<[f32; 3]> = vec![];
        let mut mesh_indicies: Vec<u32> = vec![];

        for i in 0..CHUNK_SIZE {
            let local_block_position = Chunk::local_block_position(i);
            let mut local_verticies: Vec<[f32; 3]> = CUBE_MESH
                .verticies()
                .iter()
                .map(|vertex| {
                    [
                        vertex[0] + local_block_position.x,
                        vertex[1] + local_block_position.y,
                        vertex[2] + local_block_position.z,
                    ]
                })
                .collect();
            mesh_verticies.append(local_verticies.as_mut());
            mesh_normals.append(CUBE_MESH.normals().to_owned().as_mut());

            let num_verticies = CUBE_MESH.verticies().len();
            let mut local_indicies: Vec<u32> = CUBE_MESH
                .indicies()
                .iter()
                .map(|indicie| indicie + (i * num_verticies) as u32)
                .collect();

            mesh_indicies.append(local_indicies.as_mut())
        }

        RuntimeMesh {
            verticies: mesh_verticies,
            normals: mesh_normals,
            indicies: mesh_indicies,
        }
    }

    fn local_block_position(block_index: usize) -> Position {
        let local_z = block_index / (BLOCKS_WIDTH * BLOCKS_HEIGHT);
        let block_index = block_index - (local_z * BLOCKS_WIDTH * BLOCKS_HEIGHT);
        let local_y = block_index / BLOCKS_WIDTH;
        let local_x = block_index % BLOCKS_WIDTH;

        glam::vec3(local_x as f32, local_y as f32, local_z as f32)
    }

    #[inline]
    fn world_block_position(&self, block_index: usize) -> Position {
        Chunk::local_block_position(block_index) + self.origin_position
    }
}
