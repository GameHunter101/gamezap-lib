#![allow(unused)]
use std::{
    num::NonZeroU32,
    sync::{Arc, Mutex}, collections::HashMap,
};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType, Device, Queue,
    RenderPass, SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension,
};

use crate::{model::Vertex, texture::Texture, EngineDetails};

use super::entity::EntityId;

pub struct EntityComponentGroup {
    entity: EntityId,
    normal_components: Vec<Box<dyn ComponentSystem>>,
    material_components: Vec<MaterialComponent>,
    active_material_id: Option<MaterialId>,
}

impl EntityComponentGroup {
    pub fn this_entity(&self) -> &EntityId {
        &self.entity
    }

    pub fn get_normal_components(&self) -> &Vec<Box<dyn ComponentSystem>> {
        &self.normal_components
    }

    pub fn get_material_components(&self) -> &Vec<MaterialComponent> {
        &self.material_components
    }

    pub fn get_normal_components_mut(&mut self) -> &mut Vec<Box<dyn ComponentSystem>> {
        &mut self.normal_components
    }

    pub fn get_material_components_mut(&mut self) -> &mut Vec<MaterialComponent> {
        &mut self.material_components
    }

    pub fn get_active_material_index(&self) -> &Option<MaterialId> {
        &self.active_material_id
    }

    pub fn get_active_material(&self) -> Option<&MaterialComponent> {
        if let Some(id) = &self.active_material_id {
            for material in &self.material_components {
                if material.id() == id {
                    return Some(material);
                }
            }
        }
        return None;
    }
}

pub trait ComponentSystem {
    fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        all_components: Arc<Mutex<HashMap<EntityId, Vec<Box<dyn ComponentSystem>>>>>,
    ) {
    }

    fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        all_components: Arc<Mutex<HashMap<EntityId, Vec<Box<dyn ComponentSystem>>>>>,
        engine_details: Arc<Mutex<EngineDetails>>,
        render_pass: &mut RenderPass,
    ) {
    }

    fn this_entity(&self) -> &EntityId;
}

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
        all_components: Arc<Mutex<HashMap<EntityId, Vec<Box<dyn ComponentSystem>>>>>,
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

    fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        all_components: Arc<Mutex<HashMap<EntityId, Vec<Box<dyn ComponentSystem>>>>>,
        engine_details: Arc<Mutex<EngineDetails>>,
        render_pass: &mut RenderPass,
    ) {
    }

    fn this_entity(&self) -> &EntityId {
        &self.entity
    }
}

pub type MaterialId = (String, String, usize);

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
