use std::rc::Rc;

use nalgebra as na;
use wgpu::util::DeviceExt;

use crate::materials::Material;

pub struct MeshManager {
    pub plain_pipeline_models: Vec<Rc<Mesh>>,
    pub diffuse_pipeline_models: Vec<Rc<Mesh>>,
    pub diffuse_normal_pipeline_models: Vec<Rc<Mesh>>,
}

impl MeshManager {
    pub fn init() -> Self {
        MeshManager {
            plain_pipeline_models: vec![],
            diffuse_pipeline_models: vec![],
            diffuse_normal_pipeline_models: vec![],
        }
    }
}

pub trait VertexData {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl Vertex {
    pub fn blank() -> Self {
        Vertex {
            position: [f32::MAX; 3],
            tex_coords: [f32::MAX; 2],
            normal: [f32::MAX; 3],
            bitangent: [f32::MAX; 3],
            tangent: [f32::MAX; 3],
        }
    }
}

impl VertexData for Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3, 3 => Float32x3, 4 => Float32x4];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}
#[derive(Debug)]
pub struct Models {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub transform: MeshTransform,
    pub transform_buffer: wgpu::Buffer,
    pub material_index: u32,
}

impl Mesh {
    pub fn new(
        device: &wgpu::Device,
        name: String,
        vertex_buffer: wgpu::Buffer,
        index_buffer: wgpu::Buffer,
        num_indices: u32,
        transform: MeshTransform,
        material_index: u32,
    ) -> Self {
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} transform buffer", name)),
            contents: bytemuck::cast_slice(&[transform]),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Mesh {
            name,
            vertex_buffer,
            index_buffer,
            num_indices,
            transform,
            transform_buffer,
            material_index,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshTransform {
    _transform_matrix: [[f32; 4]; 4],
}

impl MeshTransform {
    pub fn new(position: na::Vector3<f32>, rotation: na::UnitQuaternion<f32>) -> Self {
        let translation_matrix = na::Matrix4::from(na::Translation3::from(position));
        let rotation_matrix = na::Matrix4::from(rotation.to_rotation_matrix());
        let transform_matrix = translation_matrix * rotation_matrix;
        MeshTransform {
            _transform_matrix: transform_matrix.into(),
        }
    }
}

impl VertexData for Mesh {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4, 9 => Float32x3, 10 => Float32x3, 11 => Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshTransform>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}
