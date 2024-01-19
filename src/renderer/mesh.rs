use vulkano::buffer::{allocator::SubbufferAllocator, Subbuffer};

use super::vulkan::default_lit_pipeline::MeshVertex;
pub enum WindingDirection {
    Clockwise,
    CounterClockwise,
}

#[derive(Default)]
pub struct Mesh {
    vertices: Vec<MeshVertex>,
    normals: Vec<[f32; 3]>,
    indicies: Vec<u32>,
}

impl Mesh {
    pub fn add_quad(&mut self, points: [glam::Vec3; 4], winding_direction: WindingDirection) {
        let vertex_start_pos = self.vertices.len() as u32;
        let vertices: Vec<MeshVertex> = points
            .iter()
            .map(|point| MeshVertex {
                position: point.to_array(),
            })
            .collect();
        self.vertices.append(vertices.to_owned().as_mut());

        let normal = (points[1] - points[0])
            .cross(points[2] - points[0])
            .to_array();
        self.normals.push(normal);

        let indicies = match winding_direction {
            WindingDirection::Clockwise => vec![
                vertex_start_pos + 2,
                vertex_start_pos + 1,
                vertex_start_pos + 0,
                vertex_start_pos + 0,
                vertex_start_pos + 3,
                vertex_start_pos + 2,
            ],
            WindingDirection::CounterClockwise => vec![
                vertex_start_pos + 0,
                vertex_start_pos + 1,
                vertex_start_pos + 2,
                vertex_start_pos + 0,
                vertex_start_pos + 2,
                vertex_start_pos + 3,
            ],
        };

        self.indicies.append(indicies.to_owned().as_mut());
    }

    pub fn vertices(&self) -> &[MeshVertex] {
        self.vertices.as_ref()
    }

    pub fn indicies(&self) -> &[u32] {
        self.indicies.as_ref()
    }
}
