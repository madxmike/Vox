use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
    sync::Arc,
};

use glam::vec3;
use vulkano::buffer::Subbuffer;

use crate::{
    camera::Camera,
    chunk::{Chunk, CHUNK_BLOCK_DEPTH, CHUNK_BLOCK_HEIGHT, CHUNK_BLOCK_WIDTH},
    mesh::{Mesh, RuntimeMesh, StitchedMesh, WindingDirection},
    world::block_position::BlockPosition,
    world::direction::Direction,
    world::world::World,
};

use super::vulkan::{
    default_lit_pipeline::{DefaultLitIndex, DefaultLitVertex},
    mvp::MVP,
    vulkan_renderer::VulkanRenderer,
};

#[derive(Debug)]
pub enum WorldRenderError {
    CouldNotBuildTerrainMesh,
}

#[derive(Default)]
pub struct WorldRenderSystem {
    chunk_mesh_cache: HashMap<BlockPosition, StitchedMesh>,
    terrain_vertex_buffer: Option<Subbuffer<[DefaultLitVertex]>>,
    terrain_index_buffer: Option<Subbuffer<[u32]>>,
    cached_terrain_mesh: Option<StitchedMesh>,
}

impl WorldRenderSystem {
    pub fn render_world(
        &mut self,
        renderer: &mut VulkanRenderer,
        world: &World,
        camera: &Camera,
    ) -> Result<(), WorldRenderError> {
        if let None = self.cached_terrain_mesh {
            dbg!("rebuilding buffers");
            let terrain_mesh = self.build_terrain_mesh(world)?;

            self.terrain_vertex_buffer = Some(
                renderer
                    .create_vertex_buffer(terrain_mesh.verticies().iter().map(|vert| {
                        DefaultLitVertex {
                            position: vert.to_owned(),
                            normal: [0.0, 0.0, 0.0],
                        }
                    }))
                    .unwrap(),
            );

            self.terrain_index_buffer = Some(
                renderer
                    .create_index_buffer(terrain_mesh.indicies().iter().map(|idx| *idx))
                    .unwrap(),
            );

            dbg!("created buffers!");

            self.cached_terrain_mesh = Some(terrain_mesh)
        }

        let mvp = MVP {
            model: vec3(0.0, 0.0, 0.0),
            view: camera.view(),
            projection: camera.projection(),
        };

        renderer.default_lit(
            mvp,
            &self.terrain_vertex_buffer.as_ref().unwrap(),
            &self.terrain_index_buffer.as_ref().unwrap(),
        );
        Ok(())
    }

    fn build_terrain_mesh(&mut self, world: &World) -> Result<StitchedMesh, WorldRenderError> {
        let mut terrain_mesh = StitchedMesh::default();
        for (chunk_position, chunk) in &world.chunks {
            if let None = self.chunk_mesh_cache.get(chunk_position) {
                self.chunk_mesh_cache
                    .insert(chunk_position.clone(), self.build_chunk_mesh(world, chunk));
            }

            let chunk_mesh = self.chunk_mesh_cache.get(chunk_position).unwrap();

            terrain_mesh.stich(chunk_mesh)
        }

        Ok(terrain_mesh)
    }

    fn build_chunk_mesh(&self, world: &World, chunk: &Chunk) -> StitchedMesh {
        let mut chunk_mesh = StitchedMesh::default();

        let origin_position = chunk.origin_position();

        for x in 0..CHUNK_BLOCK_WIDTH {
            for y in 0..CHUNK_BLOCK_HEIGHT {
                for z in 0..CHUNK_BLOCK_DEPTH {
                    let block_position = origin_position.offset(x as i32, y as i32, z as i32);
                    let block_mesh = self.build_block_mesh(world, block_position);
                    if let None = block_mesh {
                        continue;
                    }
                    chunk_mesh.stich(&block_mesh.unwrap());
                }
            }
        }

        chunk_mesh
    }

    fn build_block_mesh(
        &self,
        world: &World,
        block_position: BlockPosition,
    ) -> Option<RuntimeMesh> {
        if let None = world.get_block_at_position(block_position) {
            return None;
        }
        let neighbors = world.get_neighbors(block_position);
        let mut block_mesh = RuntimeMesh::default();
        let block_position_vec3 = block_position.to_vec3();
        for (direction, neighbor) in neighbors.iter() {
            match (direction, neighbor) {
                (Direction::North, None) => {
                    block_mesh.add_quad(
                        [
                            glam::vec3(1.0, 0.0, 1.0) + block_position_vec3,
                            glam::vec3(0.0, 0.0, 1.0) + block_position_vec3,
                            glam::vec3(0.0, 1.0, 1.0) + block_position_vec3,
                            glam::vec3(1.0, 1.0, 1.0) + block_position_vec3,
                        ],
                        WindingDirection::Clockwise,
                    );
                }
                (Direction::South, None) => {
                    block_mesh.add_quad(
                        [
                            glam::vec3(0.0, 0.0, 0.0) + block_position_vec3,
                            glam::vec3(1.0, 0.0, 0.0) + block_position_vec3,
                            glam::vec3(1.0, 1.0, 0.0) + block_position_vec3,
                            glam::vec3(0.0, 1.0, 0.0) + block_position_vec3,
                        ],
                        WindingDirection::Clockwise,
                    );
                }
                (Direction::East, None) => {
                    block_mesh.add_quad(
                        [
                            glam::vec3(0.0, 0.0, 1.0) + block_position_vec3,
                            glam::vec3(0.0, 0.0, 0.0) + block_position_vec3,
                            glam::vec3(0.0, 1.0, 0.0) + block_position_vec3,
                            glam::vec3(0.0, 1.0, 1.0) + block_position_vec3,
                        ],
                        WindingDirection::Clockwise,
                    );
                }
                (Direction::West, None) => {
                    block_mesh.add_quad(
                        [
                            glam::vec3(1.0, 0.0, 0.0) + block_position_vec3,
                            glam::vec3(1.0, 0.0, 1.0) + block_position_vec3,
                            glam::vec3(1.0, 1.0, 1.0) + block_position_vec3,
                            glam::vec3(1.0, 1.0, 0.0) + block_position_vec3,
                        ],
                        WindingDirection::Clockwise,
                    );
                }
                (Direction::Up, None) => {
                    block_mesh.add_quad(
                        [
                            glam::vec3(0.0, 1.0, 0.0) + block_position_vec3,
                            glam::vec3(1.0, 1.0, 0.0) + block_position_vec3,
                            glam::vec3(1.0, 1.0, 1.0) + block_position_vec3,
                            glam::vec3(0.0, 1.0, 1.0) + block_position_vec3,
                        ],
                        WindingDirection::Clockwise,
                    );
                }
                (Direction::Down, None) => {
                    block_mesh.add_quad(
                        [
                            glam::vec3(0.0, 0.0, 0.0) + block_position_vec3,
                            glam::vec3(1.0, 0.0, 0.0) + block_position_vec3,
                            glam::vec3(1.0, 0.0, 1.0) + block_position_vec3,
                            glam::vec3(0.0, 0.0, 1.0) + block_position_vec3,
                        ],
                        WindingDirection::CounterClockwise,
                    );
                }
                (_, Some(_)) => {}
            }
        }

        Some(block_mesh)
    }
}
