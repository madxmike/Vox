pub trait Mesh {
    fn verticies(&self) -> &[f32];
    fn normals(&self) -> &[f32];
    fn indicies(&self) -> &[u32];
}

#[derive(Clone, Copy)]
pub struct StaticMesh<const VERTICIES_COUNT: usize, const INDICIES_COUNT: usize> {
    pub verticies: [f32; VERTICIES_COUNT],
    pub normals: [f32; VERTICIES_COUNT],
    pub indicies: [u32; INDICIES_COUNT],
}

impl<const VERTICIES_COUNT: usize, const INDICIES_COUNT: usize> Mesh
    for StaticMesh<VERTICIES_COUNT, INDICIES_COUNT>
{
    fn verticies(&self) -> &[f32] {
        &self.verticies
    }

    fn normals(&self) -> &[f32] {
        &self.normals
    }

    fn indicies(&self) -> &[u32] {
        &self.indicies
    }
}
