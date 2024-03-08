#![allow(unused)]
use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    num::NonZeroU32,
    sync::{Arc, Mutex, MutexGuard},
};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType,
    BufferDescriptor, BufferSlice, BufferUsages, Device, Queue, RenderPass, SamplerBindingType,
    ShaderStages, TextureSampleType, TextureViewDimension,
};

use nalgebra as na;

use crate::{model::Vertex, texture::Texture, EngineDetails};

use super::{
    entity::{Entity, EntityId},
    scene::Scene,
};

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub type Component = Box<dyn ComponentSystem>;

pub trait ComponentSystem: Debug {
    fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        component_map: Arc<Mutex<HashMap<EntityId, Vec<Component>>>>,
    ) {
    }

    fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        component_map: Arc<Mutex<HashMap<EntityId, Vec<Component>>>>,
        engine_details: Arc<Mutex<EngineDetails>>,
    ) {
    }

    fn render<'a: 'b, 'b>(
        &'a self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        render_pass: &mut RenderPass<'b>,
        component_map: &'a HashMap<EntityId, Vec<Component>>,
    ) {
    }
}

#[derive(Debug, PartialEq)]
pub enum ComponentType {
    Mesh,
    Transform,
    Camera,
    Custom(String),
}

#[derive(Debug)]
pub struct MeshComponent {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
}

impl MeshComponent {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        MeshComponent {
            vertices,
            indices,
            vertex_buffer: None,
            index_buffer: None,
        }
    }
}

impl ComponentSystem for MeshComponent {
    fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        component_map: Arc<Mutex<HashMap<EntityId, Vec<Component>>>>,
    ) {
        self.vertex_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Entity Vertex Buffer"),
            contents: &bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));

        self.index_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Entity Index Buffer"),
            contents: &bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        }));
    }

    fn render<'a: 'b, 'b>(
        &'a self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        render_pass: &mut RenderPass<'b>,
        component_map: &'a HashMap<EntityId, Vec<Component>>,
    ) {
        if let Some(vertex_buffer) = &self.vertex_buffer {
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        }

        let components_slice = &component_map[&0];
        let transform_component =
            Scene::find_specific_component::<TransformComponent>(components_slice);

        if let Some(comp) = transform_component {
            render_pass.set_vertex_buffer(1, comp.buffer().unwrap().slice(..));
        }

        if let Some(index_buffer) = &self.index_buffer {
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
        }
    }
}

pub type MaterialId = (String, String, usize);

#[derive(Debug)]
pub struct Material {
    vertex_shader_path: String,
    fragment_shader_path: String,
    textures: Vec<Texture>,
    enabled: bool,
    id: MaterialId,
    bind_group: BindGroup,
}

impl Material {
    pub fn new(
        vertex_shader_path: &str,
        fragment_shader_path: &str,
        textures: Vec<Texture>,
        enabled: bool,
        device: Arc<Device>,
    ) -> Self {
        let id = (
            vertex_shader_path.to_string(),
            fragment_shader_path.to_string(),
            textures.len(),
        );
        let bind_group_layout_entries = if textures.len() == 0 {
            Vec::new()
        } else {
            vec![
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(textures.len() as u32),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: NonZeroU32::new(textures.len() as u32),
                },
            ]
        };
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("Material {id:?} Bind Group Layout")),
            entries: &bind_group_layout_entries,
        });
        let views = textures.iter().map(|tex| &tex.view).collect::<Vec<_>>();

        let samplers = textures.iter().map(|tex| &tex.sampler).collect::<Vec<_>>();

        let bind_group_entries = if textures.len() == 0 {
            Vec::new()
        } else {
            vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureViewArray(&views),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::SamplerArray(&samplers),
                },
            ]
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Material {id:?} Bind Group")),
            layout: &bind_group_layout,
            entries: &bind_group_entries,
        });
        Self {
            vertex_shader_path: vertex_shader_path.to_string(),
            fragment_shader_path: fragment_shader_path.to_string(),
            textures,
            enabled,
            id,
            bind_group,
        }
    }

    pub fn id(&self) -> &MaterialId {
        &self.id
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct RawCameraData {
    view_pos: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl RawCameraData {
    fn new(position: na::Vector3<f32>, projection: na::Matrix4<f32>) -> Self {
        RawCameraData {
            view_pos: position.to_homogeneous().into(),
            view_proj: projection.into(),
        }
    }
}

#[derive(Debug)]
pub struct CameraComponent {
    entity: EntityId,
    view_proj: na::Matrix4<f32>,
}

impl CameraComponent {
    pub fn new(entity: EntityId) -> Self {
        CameraComponent {
            entity,
            view_proj: na::Matrix4::identity(),
        }
    }

    pub fn camera_bind_group_layout(device: Arc<Device>) -> BindGroupLayout {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(&format!("Default Camera Bind Group Layout")),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        bind_group_layout
    }

    pub fn create_camera_bind_group(
        &self,
        device: Arc<Device>,
        position: na::Vector3<f32>,
    ) -> BindGroup {
        let raw_camera_data = RawCameraData::new(position, self.view_proj);
        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[raw_camera_data]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &Self::camera_bind_group_layout(device.clone()),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        bind_group
    }
}

impl ComponentSystem for CameraComponent {}

#[derive(Debug)]
pub struct TransformComponent {
    matrix: na::Matrix4<f32>,
    position: na::Vector3<f32>,
    roll: f32,
    pitch: f32,
    yaw: f32,
    scale: na::Vector3<f32>,
    buf: Option<Buffer>,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            matrix: na::Matrix4::identity(),
            position: na::Vector3::zeros(),
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            scale: na::Vector3::zeros(),
            buf: None,
        }
    }
}

impl TransformComponent {
    pub fn position(&self) -> &na::Vector3<f32> {
        &self.position
    }

    pub fn update_buffer(&mut self, device: Arc<Device>) {
        let matrix = [[0.0; 4]; 4];
        let new_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("Entity Transform Buffer")),
            contents: bytemuck::cast_slice(&matrix),
            usage: BufferUsages::VERTEX,
        });
        self.buf = Some(new_buffer);
    }

    pub fn buffer(&self) -> Option<&Buffer> {
        self.buf.as_ref()
    }
}

impl ComponentSystem for TransformComponent {
    fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        component_map: Arc<Mutex<HashMap<EntityId, Vec<Component>>>>,
    ) {
        let matrix_as_arr: [[f32; 4]; 4] = self.matrix.into();
        self.buf = Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Transform Component Buffer"),
            contents: bytemuck::cast_slice(&matrix_as_arr),
            usage: BufferUsages::VERTEX,
        }));
    }
}

struct Controller {
    data: Arc<Buffer>,
}

struct Temp {
    controller: Arc<Controller>,
}

impl Temp {
    fn testing<'a: 'b, 'b>(&'a self, render_pass: &'b mut RenderPass<'b>) {
        render_pass.set_vertex_buffer(0, self.controller.data.slice(..))
    }
}
