use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, Device, Queue, RenderPass,
};

use crate::{
    ecs::component::{Component, ComponentId, ComponentSystem},
    model::Vertex,
};

use super::super::{concepts::ConceptManager, entity::EntityId, scene::AllComponents};

#[derive(Debug)]
pub struct MeshComponent {
    parent: EntityId,
    concept_ids: Vec<String>,
    id: ComponentId,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
}

impl MeshComponent {
    pub fn new(
        concept_manager: Arc<Mutex<ConceptManager>>,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
    ) -> Self {
        let mut component = MeshComponent {
            parent: EntityId::MAX,
            concept_ids: Vec::new(),
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            vertex_buffer: None,
            index_buffer: None,
        };

        let mut concepts: HashMap<String, Box<dyn Any>> = HashMap::new();
        concepts.insert("vertices".to_string(), Box::new(vertices));
        concepts.insert("indices".to_string(), Box::new(indices));

        component.register_component(concept_manager, concepts);

        component
    }
}

impl ComponentSystem for MeshComponent {
    fn register_component(
        &mut self,
        concept_manager: Arc<Mutex<ConceptManager>>,
        data: HashMap<String, Box<dyn Any>>,
    ) {
        self.concept_ids = data.keys().cloned().collect();

        let mut concept_manager = concept_manager.lock().unwrap();

        concept_manager.register_component_concepts(self.id, data);
    }

    fn initialize(
        &mut self,
        device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: AllComponents,
        concept_manager: Arc<Mutex<ConceptManager>>,
    ) {
        let concept_manager = concept_manager.lock().unwrap();
        let vertices = concept_manager
            .get_concept::<Vec<Vertex>>(self.id, "vertices".to_string())
            .unwrap();

        self.vertex_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Entity Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));

        let indices = concept_manager
            .get_concept::<Vec<u32>>(self.id, "indices".to_string())
            .unwrap();

        self.index_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Entity Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        }));
    }

    fn render<'a: 'b, 'b>(
        &'a self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        render_pass: &mut RenderPass<'b>,
        _component_map: &'a HashMap<EntityId, Vec<Component>>,
        concept_manager: &'a ConceptManager,
    ) {
        let vertex_buffer = self.vertex_buffer.as_ref();
        if let Some(vertex_buffer) = &vertex_buffer {
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        }

        let index_buffer = self.index_buffer.as_ref();
        let indices = concept_manager
            .get_concept::<Vec<u32>>(self.id, "indices".to_string())
            .unwrap();

        if let Some(index_buffer) = index_buffer {
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn update_metadata(&mut self, parent: EntityId, same_component_count: u32) {
        self.parent = parent;
        self.id.0 = parent;
        self.id.2 = same_component_count;
    }

    fn get_parent_entity(&self) -> EntityId {
        self.parent
    }

    fn get_id(&self) -> ComponentId {
        self.id
    }
}
