use std::borrow::BorrowMut;

pub trait Mesh {
    fn verticies(&self) -> &[glam::Vec3];
    fn normals(&self) -> &[glam::Vec3];
    fn indicies(&self) -> &[u32];
}

#[derive(Clone, Debug, Default)]
pub struct RuntimeMesh {
    pub verticies: Vec<glam::Vec3>,
    pub normals: Vec<glam::Vec3>,
    pub indicies: Vec<u32>,
}

impl RuntimeMesh {
    pub fn add_quad(&mut self, points: [glam::Vec3; 4]) {
        self.verticies.append(points.to_vec().borrow_mut());

        // TODO (Michael): We can calculate these normals from the verts
        self.normals.append(
            [
                glam::vec3(0.0, 0.0, 0.0),
                glam::vec3(0.0, 0.0, 0.0),
                glam::vec3(0.0, 0.0, 0.0),
                glam::vec3(0.0, 0.0, 0.0),
            ]
            .to_vec()
            .borrow_mut(),
        );

        let num_existing_indicies = self.indicies.len() as u32;
        self.indicies.append(
            [
                num_existing_indicies + 2,
                num_existing_indicies + 1,
                num_existing_indicies,
                num_existing_indicies,
                num_existing_indicies + 3,
                num_existing_indicies + 2,
            ]
            .to_vec()
            .borrow_mut(),
        );
    }
}

impl Mesh for RuntimeMesh {
    fn verticies(&self) -> &[glam::Vec3] {
        &self.verticies
    }

    fn normals(&self) -> &[glam::Vec3] {
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
    pub verticies: Vec<glam::Vec3>,
    pub normals: Vec<glam::Vec3>,
    pub indicies: Vec<u32>,
}

impl StitchedMesh {
    pub fn stich(&mut self, mesh: &Box<dyn Mesh>) {
        let num_existing_verticies = self.verticies.len();

        let mesh_verticies = mesh.verticies();

        let stiched_verticies: &mut Vec<glam::Vec3> = &mut self.verticies;
        stiched_verticies.append(&mut mesh_verticies.to_vec());

        let stiched_normals: &mut Vec<glam::Vec3> = &mut self.normals;
        stiched_normals.append(&mut mesh.normals().to_vec());

        dbg!(mesh.verticies());
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
    fn verticies(&self) -> &[glam::Vec3] {
        &self.verticies
    }

    fn normals(&self) -> &[glam::Vec3] {
        &self.normals
    }

    fn indicies(&self) -> &[u32] {
        &self.indicies
    }
}
