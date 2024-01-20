use std::ops::Deref;
use std::sync::mpsc::{channel, Receiver};
use std::{collections::HashMap, time::Instant};

use glam::vec3;
use rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, ParallelBridge, ParallelIterator,
};
use tokio::sync::RwLock;
use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::CopyBufferInfo;
use vulkano::sync::HostAccessError;

use crate::world::block::Block;
use crate::world::chunk::{Chunk, CHUNK_BLOCK_DEPTH, CHUNK_BLOCK_HEIGHT, CHUNK_BLOCK_WIDTH};
use crate::world::{block_position, chunk};
use crate::{
    camera::Camera, world::block_position::BlockPosition, world::direction::Direction,
    world::world::World,
};

use super::chunk_mesher::ChunkMesher;
use super::staged_buffer::StagedBuffer;
use super::{
    mesh::{Mesh, WindingDirection},
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
                .create_staged_vertex_buffer::<MeshVertex>(1024 * 1024 * 160),
            opaque_chunk_index_buffer: renderer
                .create_staged_index_buffer::<u32>(1024 * 1024 * 160),
        }
    }
    pub fn build_chunk_meshes(&mut self, world: &World) {
        for (_, chunk) in world.chunks.iter() {
            self.chunk_mesher.begin_meshing_chunk(chunk.to_owned())
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
            // TODO (Michael): We have to uncouple the buffers on the gpu and cpu, otherwise this can only be written too once
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
