use std::{
    borrow::BorrowMut,
    cell::{BorrowMutError, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    camera::Camera,
    mesh::{Mesh, RuntimeMesh, StaticMesh, StitchedMesh},
    renderer::Renderer,
};

use super::{block_position::BlockPosition, chunk::Chunk, world::World};

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
            let chunk_mesh = self
                .chunk_mesh_cache
                .entry(chunk_position.clone())
                .or_insert_with(|| chunk.build_chunk_mesh());
            terrain_mesh.stich(chunk_mesh.clone())
        }

        Ok(Box::new(terrain_mesh))
    }
}
