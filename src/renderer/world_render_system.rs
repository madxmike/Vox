use std::{
    borrow::BorrowMut,
    cell::{BorrowMutError, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    camera::Camera,
    chunk::{Chunk, CHUNK_BLOCK_DEPTH, CHUNK_BLOCK_HEIGHT, CHUNK_BLOCK_WIDTH},
    mesh::{Mesh, RuntimeMesh, StitchedMesh, WindingDirection},
    world::block::{self, Block},
    world::block_position::{self, BlockPosition},
    world::direction::Direction,
    world::world::World,
};

use super::renderer::{self, Renderer};

#[derive(Debug)]
pub enum WorldRenderError {
    CouldNotBuildTerrainMesh,
}

#[derive(Default)]
pub struct WorldRenderSystem {
    chunk_mesh_cache: HashMap<BlockPosition, StitchedMesh>,
}

impl WorldRenderSystem {
    pub fn render_world(
        &mut self,
        renderer: &mut Box<dyn Renderer>,
        world: &World,
        camera: &Camera,
    ) -> Result<(), WorldRenderError> {
        let terrain_mesh = self.build_terrain_mesh(world)?;

        renderer.as_mut().render(camera, terrain_mesh);

        Ok(())
    }

    fn build_terrain_mesh(&mut self, world: &World) -> Result<Box<dyn Mesh>, WorldRenderError> {
        let mut terrain_mesh = StitchedMesh::default();
        for (chunk_position, chunk) in &world.chunks {
            if let None = self.chunk_mesh_cache.get(chunk_position) {
                self.chunk_mesh_cache
                    .insert(chunk_position.clone(), self.build_chunk_mesh(world, chunk));
            }

            let chunk_mesh = self.chunk_mesh_cache.get(chunk_position).unwrap();

            terrain_mesh.stich(chunk_mesh)
        }

        Ok(Box::new(terrain_mesh))
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
