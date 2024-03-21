use nalgebra as na;

pub trait VertexData {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn blank() -> Self {
        Vertex {
            position: [f32::MAX; 3],
            tex_coords: [f32::MAX; 2],
            normal: [f32::MAX; 3],
        }
    }

    pub fn translate(mut self, vector: na::Vector3<f32>) -> Self {
        self.position[0] += vector.x;
        self.position[1] += vector.y;
        self.position[2] += vector.z;
        self
    }

    pub fn matrix_mult(mut self, matrix: na::Matrix3<f32>) -> Self {
        let vector = matrix * na::Vector3::from(self.position);
        self.position = vector.into();
        self
    }
}

impl VertexData for Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}
