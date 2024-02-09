#![allow(unused)]
use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    num::NonZeroU32,
    sync::{Arc, Mutex},
};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages,
    Device, Queue, RenderPass, SamplerBindingType, ShaderStages, TextureSampleType,
    TextureViewDimension,
};

use nalgebra as na;

use crate::{model::Vertex, texture::Texture, EngineDetails};

use super::entity::{Entity, EntityId};

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
        all_entities: Arc<Mutex<Vec<Entity>>>,
    ) {
    }

    fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        all_components: Arc<Mutex<Vec<Entity>>>,
        engine_details: Arc<Mutex<EngineDetails>>,
        render_pass: &mut RenderPass,
    ) {
    }

    fn this_entity(&self) -> &EntityId;
}

#[derive(Debug)]
pub struct MeshComponent {
    entity: EntityId,
    vertices: Vec<Vertex>,
    indices: Vec<u64>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
}

impl ComponentSystem for MeshComponent {
    fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        all_entities: Arc<Mutex<Vec<Entity>>>,
    ) {
        self.vertex_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("Entity {:?} Vertex Buffer", self.entity)),
            contents: &bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));

        self.index_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("Entity {:?} Vertex Buffer", self.entity)),
            contents: &bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        }));
    }

    fn this_entity(&self) -> &EntityId {
        &self.entity
    }
}

pub type MaterialId = (String, String, usize);

#[derive(Debug)]
pub struct MaterialComponent {
    entity: EntityId,
    vertex_shader_path: String,
    fragment_shader_path: String,
    textures: Vec<Texture>,
    enabled: bool,
    id: MaterialId,
    bind_group: BindGroup,
}

impl MaterialComponent {
    pub fn new(
        entity: EntityId,
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
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("Material {id:?} Bind Group Layout")),
            entries: &[
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
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Material {id:?} Bind Group")),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureViewArray(
                        &textures
                            .iter()
                            .map(|tex| &tex.view)
                            .collect::<Vec<_>>()
                            .as_slice(),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::SamplerArray(
                        &textures
                            .iter()
                            .map(|tex| &tex.sampler)
                            .collect::<Vec<_>>()
                            .as_slice(),
                    ),
                },
            ],
        });
        Self {
            entity,
            vertex_shader_path: vertex_shader_path.to_string(),
            fragment_shader_path: fragment_shader_path.to_string(),
            textures,
            enabled,
            id,
            bind_group,
        }
    }

    pub fn this_entity(&self) -> &EntityId {
        &self.entity
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
            label: Some(&format!("{:?} Camera Buffer", self.entity)),
            contents: bytemuck::cast_slice(&[raw_camera_data]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some(&format!("{:?} Camera Bind Group", self.entity)),
            layout: &Self::camera_bind_group_layout(device.clone()),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: todo!(),
            }],
        });
        bind_group
    }
}

impl ComponentSystem for CameraComponent {
    fn this_entity(&self) -> &EntityId {
        &self.entity
    }
}

#[derive(Debug)]
pub struct TransformComponent {
    entity: EntityId,
    position: na::Vector3<f32>,
    roll: f32,
    pitch: f32,
    yaw: f32,
    scale: na::Vector3<f32>,
}

impl TransformComponent {
    pub fn position(&self) -> &na::Vector3<f32> {
        &self.position
    }
}

impl ComponentSystem for TransformComponent {
    fn this_entity(&self) -> &EntityId {
        &self.entity
    }
}
