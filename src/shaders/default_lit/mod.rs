use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/default_lit/vert.glsl",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/default_lit/frag.glsl",
    }
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct DefaultLitVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
}
