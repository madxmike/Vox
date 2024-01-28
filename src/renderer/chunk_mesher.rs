use std::sync::mpsc::{channel, Receiver, Sender};

use crate::world::{
    block::Block,
    block_position::BlockPosition,
    chunk::{Chunk, CHUNK_BLOCK_DEPTH, CHUNK_BLOCK_HEIGHT, CHUNK_BLOCK_WIDTH},
    direction::{Direction},
};

use super::mesh::{Mesh, WindingDirection};

pub struct ChunkMesher {
    ready_chunk_meshes_tx: Sender<(BlockPosition, Mesh)>,
    ready_chunk_meshes_rx: Receiver<(BlockPosition, Mesh)>,
}

impl ChunkMesher {
    pub fn new() -> Self {
        let (tx, rx) = channel::<(BlockPosition, Mesh)>();
        ChunkMesher {
            ready_chunk_meshes_tx: tx,
            ready_chunk_meshes_rx: rx,
        }
    }

    pub fn begin_meshing_chunk(&self, chunk: Chunk, neighbor_chunks: Vec<Option<Chunk>>) {
        let tx = self.ready_chunk_meshes_tx.clone();
        tokio_rayon::spawn(move || {
            let chunk_mesh = ChunkMesher::mesh_chunk(&chunk, &neighbor_chunks);
            if !chunk_mesh.is_empty() {
                tx.send((chunk.origin_position(), chunk_mesh));
            }
        });
    }

    pub fn ready_chunk_meshes(&self) -> Vec<(BlockPosition, Mesh)> {
        let mut ready_chunk_meshes = Vec::new();

        loop {
            let chunk_mesh_result = self.ready_chunk_meshes_rx.try_recv();
            match chunk_mesh_result {
                Ok(chunk_mesh) => ready_chunk_meshes.push(chunk_mesh),
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    panic!("chunk mesher ready chunk mesher rx disconnected")
                }
            }
        }

        ready_chunk_meshes
    }

    fn mesh_chunk(chunk: &Chunk, neighbor_chunks: &[Option<Chunk>]) -> Mesh {
        let mut chunk_mesh = Mesh::default();

        let origin_position = chunk.origin_position();

        for x in 0..CHUNK_BLOCK_WIDTH {
            for y in 0..CHUNK_BLOCK_HEIGHT {
                for z in 0..CHUNK_BLOCK_DEPTH {
                    let block_position = origin_position.offset(x as i32, y as i32, z as i32);
                    if let Some(_block) = chunk.get_block_at_position(block_position).ok() {
                        ChunkMesher::mesh_block(
                            chunk,
                            neighbor_chunks,
                            &mut chunk_mesh,
                            block_position,
                        );
                    }
                }
            }
        }

        chunk_mesh
    }

    fn mesh_block(
        chunk: &Chunk,
        neighbor_chunks: &[Option<Chunk>],
        mesh: &mut Mesh,
        block_position: BlockPosition,
    ) {
        let neighbors = ChunkMesher::get_neighbors(chunk, neighbor_chunks, block_position);
        let block_position_vec3 = block_position.to_vec3();

        for (direction, neighbor) in neighbors.iter() {
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

    /// Gets the 6 neighbor blocks of a block at the position.
    /// If a block is air, the neighbor block will be [None].
    /// This will always be in [Direction] order.
    pub fn get_neighbors(
        chunk: &Chunk,
        neighbor_chunks: &[Option<Chunk>],
        position: BlockPosition,
    ) -> Vec<(Direction, Option<Block>)> {
        let offsets: Vec<(Direction, (i32, i32, i32))> = vec![
            (Direction::North, (0, 0, 1)),
            (Direction::South, (0, 0, -1)),
            (Direction::East, (-1, 0, 0)),
            (Direction::West, (1, 0, 0)),
            (Direction::Up, (0, 1, 0)),
            (Direction::Down, (0, -1, 0)),
        ];

        let mut neighbors = vec![];
        'outer: for (direction, offset) in offsets {
            match chunk.get_block_at_position(position.offset(offset.0, offset.1, offset.2)) {
                Ok(block) => {
                    neighbors.push((direction, block));
                    continue;
                }
                Err(_) => {}
            }

            for neighbor_chunk in neighbor_chunks {
                if let None = neighbor_chunk {
                    continue;
                }

                match neighbor_chunk
                    .as_ref()
                    .unwrap()
                    .get_block_at_position(position.offset(offset.0, offset.1, offset.2))
                {
                    Ok(block) => {
                        neighbors.push((direction, block));
                        continue 'outer;
                    }
                    Err(_) => continue,
                }
            }

            neighbors.push((direction, None))
        }

        neighbors
    }
}
