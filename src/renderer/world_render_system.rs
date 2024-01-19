

use glam::vec3;
use vulkano::buffer::{Subbuffer};

use crate::{
    camera::Camera,
    chunk::{Chunk, CHUNK_BLOCK_DEPTH, CHUNK_BLOCK_HEIGHT, CHUNK_BLOCK_WIDTH},
    world::block_position::BlockPosition,
    world::direction::Direction,
    world::world::World,
};

use super::{
    mesh::{Mesh, WindingDirection},
    vulkan::{
        default_lit_pipeline::{MeshVertex},
        mvp::MVP,
        vulkan_renderer::VulkanRenderer,
    },
};

#[derive(Debug)]
pub enum WorldRenderError {
    CouldNotBuildTerrainMesh,
}

pub struct WorldRenderSystem {
    opaque_chunk_meshes: Vec<Mesh>,
    opaque_chunk_vertex_buffer: Subbuffer<[MeshVertex]>,
    opaque_chunk_index_buffer: Subbuffer<[u32]>,
}

impl WorldRenderSystem {
    pub fn new(renderer: &VulkanRenderer) -> Self {
        WorldRenderSystem {
            opaque_chunk_meshes: Default::default(),
            opaque_chunk_vertex_buffer: renderer
                .create_sized_vertex_buffer::<MeshVertex>(1024 * 1024 * 50)
                .unwrap(),
            opaque_chunk_index_buffer: renderer
                .create_sized_index_buffer::<u32>(1024 * 1024 * 50)
                .unwrap(),
        }
    }
    pub fn build_chunk_meshes(&mut self, world: &World) {
        for chunk in world.chunks.iter() {
            let chunk_mesh = self.build_chunk_mesh(world, chunk.1);
            if chunk_mesh.vertices().len() != 0 {
                self.opaque_chunk_meshes.push(chunk_mesh);
            }
        }

        self.write_meshes();
    }

    pub fn write_meshes(&mut self) {
        let mut vertex_writer = self.opaque_chunk_vertex_buffer.write().unwrap();
        let mut vertex_writer_iter = vertex_writer.iter_mut();

        let mut index_offset = 0;
        let mut index_writer = self.opaque_chunk_index_buffer.write().unwrap();
        let mut index_writer_iter = index_writer.iter_mut();
        for ocm in self.opaque_chunk_meshes.iter() {
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
    }

    pub fn render_world(
        &mut self,
        renderer: &mut VulkanRenderer,
        _world: &World,
        camera: &Camera,
    ) -> Result<(), WorldRenderError> {
        let mvp = MVP {
            model: vec3(0.0, 0.0, 0.0),
            view: camera.view(),
            projection: camera.projection(),
        };

        renderer.default_lit(
            mvp,
            &self.opaque_chunk_vertex_buffer,
            &self.opaque_chunk_index_buffer,
        );
        Ok(())
    }

    fn build_chunk_mesh(&self, world: &World, chunk: &Chunk) -> Mesh {
        let mut chunk_mesh = Mesh::default();

        let origin_position = chunk.origin_position();

        for x in 0..CHUNK_BLOCK_WIDTH {
            for y in 0..CHUNK_BLOCK_HEIGHT {
                for z in 0..CHUNK_BLOCK_DEPTH {
                    let block_position = origin_position.offset(x as i32, y as i32, z as i32);
                    self.build_block_mesh(world, &mut chunk_mesh, block_position);
                }
            }
        }

        chunk_mesh
    }

    fn build_block_mesh(&self, world: &World, mesh: &mut Mesh, block_position: BlockPosition) {
        if let None = world.get_block_at_position(block_position) {
            return;
        }

        let neighbors = world.get_neighbors(block_position);
        let block_position_vec3 = block_position.to_vec3();

        let mut n = 0;
        for (direction, neighbor) in neighbors.iter() {
            if let Some(_x) = neighbor {
                n += 1;
                // dbg!(direction);
            }
            match (direction, neighbor) {
                (Direction::North, None) => {
                    mesh.add_quad(
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
                    mesh.add_quad(
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
                    mesh.add_quad(
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
                    mesh.add_quad(
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
                    mesh.add_quad(
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
                    mesh.add_quad(
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
    }
}
