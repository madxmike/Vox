use std::{
    borrow::BorrowMut,
    cell::{BorrowMutError, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    camera::Camera,
    mesh::{Mesh, RuntimeMesh, StitchedMesh},
    renderer::Renderer,
};

use super::{block_position::BlockPosition, chunk::Chunk, world::World};

#[derive(Debug)]
pub enum WorldRenderError {
    CouldNotBuildTerrainMesh,
}

#[derive(Default)]
pub struct WorldRenderSystem {
    chunk_mesh_cache: HashMap<BlockPosition, Box<dyn Mesh>>,
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
                    .insert(chunk_position.clone(), self.build_chunk_mesh(world));
            }

            let chunk_mesh = self.chunk_mesh_cache.get(chunk_position).unwrap();

            terrain_mesh.stich(chunk_mesh)
        }

        Ok(Box::new(terrain_mesh))
    }

    fn build_chunk_mesh(&self, world: &World) -> Box<dyn Mesh> {
        dbg!("building chunk mesh!");
        let mut runtime_mesh = RuntimeMesh::default();
        runtime_mesh.add_quad([
            glam::vec3(1.0, 0.0, 1.0),
            glam::vec3(0.0, 0.0, 1.0),
            glam::vec3(0.0, 1.0, 1.0),
            glam::vec3(1.0, 1.0, 1.0),
        ]);
        Box::new(runtime_mesh)
    }
}
