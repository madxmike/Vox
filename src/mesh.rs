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

#[derive(Clone)]
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
