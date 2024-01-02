use super::chunk::Chunk;

pub struct World {
    // TODO (Michael): Later we want to move this to a chunk pool when we do loading / unloading
    pub chunks: Vec<Chunk>,
}
