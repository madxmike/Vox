use std::borrow::BorrowMut;

pub trait Mesh {
    fn verticies(&self) -> &[[f32; 3]];
    fn normals(&self) -> &[[f32; 3]];
    fn indicies(&self) -> &[u32];
}

#[derive(Clone, Copy)]
pub struct StaticMesh<const VERTICIES_COUNT: usize, const INDICIES_COUNT: usize> {
    pub verticies: [[f32; 3]; VERTICIES_COUNT],
    pub normals: [[f32; 3]; VERTICIES_COUNT],
    pub indicies: [u32; INDICIES_COUNT],
}

impl<const VERTICIES_COUNT: usize, const INDICIES_COUNT: usize> Mesh
    for StaticMesh<VERTICIES_COUNT, INDICIES_COUNT>
{
    fn verticies(&self) -> &[[f32; 3]] {
        &self.verticies
    }

    fn normals(&self) -> &[[f32; 3]] {
        &self.normals
    }

    fn indicies(&self) -> &[u32] {
        &self.indicies
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeMesh {
    pub verticies: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indicies: Vec<u32>,
}

impl Mesh for RuntimeMesh {
    fn verticies(&self) -> &[[f32; 3]] {
        &self.verticies
    }

    fn normals(&self) -> &[[f32; 3]] {
        &self.normals
    }

    fn indicies(&self) -> &[u32] {
        &self.indicies
    }
}

/// A [StichedMesh] is a [Mesh] stitched together from other meshes (such as from multiple [StaticMeshs](StaticMesh)).
/// Basically, it acts as a easy way to build one big mess out of many smaller meshes.
/// This prevents having to duplicate this logic everywhere we do it.
/// This does not try to do any combining of faces or other deduplication logic.
#[derive(Default, Clone)]
pub struct StitchedMesh {
    pub verticies: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indicies: Vec<u32>,
}

impl StitchedMesh {
    pub fn stich(&mut self, mesh: impl Mesh) {
        let num_existing_verticies = self.verticies.len();

        let mesh_verticies = mesh.verticies();

        let stiched_verticies: &mut Vec<[f32; 3]> = &mut self.verticies;
        stiched_verticies.append(&mut mesh_verticies.to_vec());

        let stiched_normals: &mut Vec<[f32; 3]> = &mut self.normals;
        stiched_normals.append(&mut mesh.normals().to_vec());

        // Move all the indicies up by the exisiting vertex count before stitching.
        // This is to ensure that the index will still refer to the same vertex
        // once everything is added together. I.e. if there are 30 existing verticies
        // and the index refers to vertex 3 it will now refer to vertex 33.
        let mesh_indicies: Vec<u32> = mesh
            .indicies()
            .iter()
            .map(|index| index + num_existing_verticies as u32)
            .collect();

        let stiched_indicies: &mut Vec<u32> = &mut self.indicies;
        stiched_indicies.append(&mut mesh_indicies.to_vec());
    }
}

impl Mesh for StitchedMesh {
    fn verticies(&self) -> &[[f32; 3]] {
        &self.verticies
    }

    fn normals(&self) -> &[[f32; 3]] {
        &self.normals
    }

    fn indicies(&self) -> &[u32] {
        &self.indicies
    }
}
