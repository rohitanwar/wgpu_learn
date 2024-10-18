#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

// Changed
pub const VERTICES: &[Vertex] = &[
    // Changed
    Vertex { position: [-1.0, -3.0, 0.0], tex_coords: [0.0, 3.0], }, // A
    Vertex { position: [-1.0,  1.0, 0.0], tex_coords: [0.0, 0.0], }, // B
    Vertex { position: [ 3.0,  1.0, 0.0], tex_coords: [3.0, 0.0], }, // C
    //Vertex { position: [ 1.0, -1.0, 0.0], tex_coords: [1.0, 1.0], }, // D
];
 

pub const INDICES: &[u16] = &[
    0, 1, 2,
    //0, 2, 3,
];


impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // NEW!
                },
            ]
        }
    }
}

