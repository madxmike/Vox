

use std::{collections::HashMap};

use glam::vec3;
use rayon::iter::{
    ParallelIterator,
};



use vulkano::sync::HostAccessError;


use crate::world::chunk::{CHUNK_BLOCK_DEPTH, CHUNK_BLOCK_HEIGHT, CHUNK_BLOCK_WIDTH};

use crate::{
    camera::Camera, world::block_position::BlockPosition,
    world::world::World,
};

use super::chunk_mesher::ChunkMesher;
use super::staged_buffer::StagedBuffer;
use super::{
    mesh::{Mesh},
    vulkan::{default_lit_pipeline::MeshVertex, mvp::MVP, vulkan_renderer::VulkanRenderer},
};

pub struct WorldRenderSystem {
    chunk_mesher: ChunkMesher,

    opaque_chunk_meshes: HashMap<BlockPosition, Mesh>,
    opaque_chunk_vertex_buffer: StagedBuffer<MeshVertex>,
    opaque_chunk_index_buffer: StagedBuffer<u32>,
}

impl WorldRenderSystem {
    pub fn new(renderer: &VulkanRenderer) -> Self {
        WorldRenderSystem {
            chunk_mesher: ChunkMesher::new(),
            opaque_chunk_meshes: Default::default(),
            opaque_chunk_vertex_buffer: renderer
                .create_staged_vertex_buffer::<MeshVertex>(32 << 20),
            opaque_chunk_index_buffer: renderer.create_staged_index_buffer::<u32>(48 << 20),
        }
    }
    pub fn build_chunk_meshes(&mut self, world: &World) {
        for (chunk_origin_position, chunk) in world.chunks.iter() {
            let neighbor_chunks = vec![
                world
                    .chunks
                    .get(&chunk_origin_position.offset(0, 0, CHUNK_BLOCK_DEPTH as i32))
                    .copied(),
                world
                    .chunks
                    .get(&chunk_origin_position.offset(0, 0, -(CHUNK_BLOCK_DEPTH as i32)))
                    .copied(),
                world
                    .chunks
                    .get(&chunk_origin_position.offset(CHUNK_BLOCK_WIDTH as i32, 0, 0))
                    .copied(),
                world
                    .chunks
                    .get(&chunk_origin_position.offset(-(CHUNK_BLOCK_WIDTH as i32), 0, 0))
                    .copied(),
                world
                    .chunks
                    .get(&chunk_origin_position.offset(0, CHUNK_BLOCK_HEIGHT as i32, 0))
                    .copied(),
                world
                    .chunks
                    .get(&chunk_origin_position.offset(0, -(CHUNK_BLOCK_HEIGHT as i32), 0))
                    .copied(),
            ];
            self.chunk_mesher
                .begin_meshing_chunk(chunk.to_owned(), neighbor_chunks)
        }
    }

    pub fn write_meshes(&mut self) -> Result<(), HostAccessError> {
        let mut vertex_writer = self.opaque_chunk_vertex_buffer.write()?;
        let mut index_writer = self.opaque_chunk_index_buffer.write()?;

        let mut vertex_writer_iter = vertex_writer.iter_mut();
        let mut index_writer_iter = index_writer.iter_mut();

        let mut index_offset = 0;

        for (_, ocm) in self.opaque_chunk_meshes.iter() {
            for vertex in ocm.vertices() {
                let existing =  vertex_writer_iter.next().expect("tried to write mesh to vertex buffer, but vertex buffer has no room available!");
                *existing = *vertex;
            }

            for index in ocm.indicies() {
                let existing = index_writer_iter.next().expect(
                    "tried to write mesh to index buffer, but index buffer has no room available!",
                );
                *existing = *index + index_offset;
            }

            index_offset += ocm.vertices().len() as u32;
        }

        Ok(())
    }

    pub fn render_world(&mut self, renderer: &mut VulkanRenderer, camera: &Camera) {
        let chunk_meshes = self.chunk_mesher.ready_chunk_meshes();

        if !chunk_meshes.is_empty() {
            for (chunk_origin_pos, chunk_mesh) in chunk_meshes {
                self.opaque_chunk_meshes
                    .insert(chunk_origin_pos, chunk_mesh);
            }
            // TODO (Michael): Instead of having meshes and then copying to a vertex buffer, we should just create the mesh directly in the vertex buffer memory
            self.write_meshes();
            self.opaque_chunk_vertex_buffer.upload_to_device(renderer);
            self.opaque_chunk_index_buffer.upload_to_device(renderer);
        }

        let mvp = MVP {
            model: vec3(0.0, 0.0, 0.0),
            view: camera.view(),
            projection: camera.projection(),
        };

        renderer.default_lit(
            mvp,
            &self.opaque_chunk_vertex_buffer.device_buffer(),
            &self.opaque_chunk_index_buffer.device_buffer(),
        );
    }
}
