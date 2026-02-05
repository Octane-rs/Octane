use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(super) struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
}

pub(super) const VERTICES: &[Vertex] = &[
    // BL
    Vertex {
        position: [-1.0, -1.0, 0.0],
        tex_coord: [0.0, 1.0],
    },
    // TL
    Vertex {
        position: [1.0, -1.0, 0.0],
        tex_coord: [1.0, 1.0],
    },
    // BR
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
    // TL
    Vertex {
        position: [-1.0, -1.0, 0.0],
        tex_coord: [0.0, 1.0],
    },
    // TR
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
    // BR
    Vertex {
        position: [-1.0, 1.0, 0.0],
        tex_coord: [0.0, 0.0],
    },
];
