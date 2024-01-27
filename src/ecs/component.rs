use std::sync::{Arc, Mutex};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Device, Queue,
};

use crate::{model::Vertex, texture::Texture, EngineDetails};

use super::entity::EntityId;

pub struct EntityComponentGroup {
    entity: EntityId,
    normal_components: Vec<Box<dyn ComponentSystem>>,
    material_components: Vec<MaterialComponent>,
    active_material: Option<MaterialId>,
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

    pub fn get_active_material(&self) -> &Option<MaterialId> {
        &self.active_material
    }
}

pub trait ComponentSystem {
    fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        entity_components: Arc<Mutex<EntityComponentGroup>>,
    ) {
    }

    fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        entity_components: Arc<Mutex<EntityComponentGroup>>,
        engine_details: Arc<Mutex<EngineDetails>>,
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
        entity_components: Arc<Mutex<EntityComponentGroup>>,
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

pub struct MaterialComponent {
    entity: EntityId,
    vertex_shader_path: String,
    fragment_shader_path: String,
    textures: Vec<Texture>,
    enabled: bool,
    id: MaterialId,
}

impl MaterialComponent {
    pub fn new(
        entity: EntityId,
        vertex_shader_path: &str,
        fragment_shader_path: &str,
        textures: Vec<Texture>,
        enabled: bool,
    ) -> Self {
        let id = (
            vertex_shader_path.to_string(),
            fragment_shader_path.to_string(),
            textures.len(),
        );
        Self {
            entity,
            vertex_shader_path: vertex_shader_path.to_string(),
            fragment_shader_path: fragment_shader_path.to_string(),
            textures,
            enabled,
            id,
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
}
