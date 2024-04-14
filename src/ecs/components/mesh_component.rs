use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    io::{BufReader, Cursor},
    rc::Rc,
    sync::{Arc, Mutex},
};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, Device, Queue, RenderPass,
};

use crate::{
    ecs::component::{Component, ComponentId, ComponentSystem},
    model::Vertex,
    EngineDetails, EngineSystems,
};

use super::super::{concepts::ConceptManager, entity::EntityId, scene::AllComponents};

#[derive(Debug)]
pub enum MeshComponentError {
    FailedToLoadObj,
    FailedToLoadMtl,
}

#[derive(Debug, Clone)]
pub struct MeshComponent {
    parent: EntityId,
    concept_ids: Vec<String>,
    id: ComponentId,
    vertex_buffer: Arc<Option<Buffer>>,
    index_buffer: Arc<Option<Buffer>>,
}

impl MeshComponent {
    pub fn new(
        concept_manager: Rc<Mutex<ConceptManager>>,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
    ) -> Self {
        let mut component = MeshComponent {
            parent: EntityId::MAX,
            concept_ids: Vec::new(),
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            vertex_buffer: Arc::new(None),
            index_buffer: Arc::new(None),
        };

        let mut concepts: HashMap<String, Box<dyn Any>> = HashMap::new();
        concepts.insert("vertices".to_string(), Box::new(vertices));
        concepts.insert("indices".to_string(), Box::new(indices));

        component.register_component(concept_manager, concepts);

        component
    }

    pub async fn from_obj(
        concept_manager: Rc<Mutex<ConceptManager>>,
        obj_path: &str,
        expect_material: bool,
    ) -> Result<(), MeshComponentError> {
        let obj_cursor = Cursor::new(obj_path);
        let mut obj_reader = BufReader::new(obj_cursor);
        let obj_load_res = tobj::load_obj_buf_async(
            &mut obj_reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            |p| async move { tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(p))) },
        )
        .await;

        if let Ok((models, materials_res)) = obj_load_res {
            if materials_res.is_err() && expect_material {
                return Err(MeshComponentError::FailedToLoadMtl);
            }

            let materials = materials_res.unwrap_or(vec![tobj::Material::default()]);

            for (i, m) in models.iter().enumerate() {
                let mesh = &m.mesh;

                // let mut verts = Vec::with_capacity(mesh.positions.len());
                /* for index in &mesh.positions {
                    verts.push(vert)
                } */
            }

            return Ok(());
        }
        Err(MeshComponentError::FailedToLoadObj)
    }
}

impl ComponentSystem for MeshComponent {
    fn register_component(
        &mut self,
        concept_manager: Rc<Mutex<ConceptManager>>,
        data: HashMap<String, Box<dyn Any>>,
    ) {
        self.concept_ids = data.keys().cloned().collect();

        concept_manager
            .lock()
            .unwrap()
            .register_component_concepts(self.id, data);
    }

    fn initialize(
        &mut self,
        device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &AllComponents,
        concept_manager: Rc<Mutex<ConceptManager>>,
    ) {
        let concept_manager = concept_manager.lock().unwrap();
        let vertices = concept_manager
            .get_concept::<Vec<Vertex>>(self.id, "vertices".to_string())
            .unwrap();

        self.vertex_buffer = Arc::new(Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Entity Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })));

        let indices = concept_manager
            .get_concept::<Vec<u32>>(self.id, "indices".to_string())
            .unwrap();

        self.index_buffer = Arc::new(Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Entity Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        })));
    }

    fn render<'a: 'b, 'b>(
        &'a self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        render_pass: &mut RenderPass<'b>,
        _component_map: &'a HashMap<EntityId, Vec<Component>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: &EngineDetails,
        _engine_systems: &EngineSystems,
    ) {
        let concept_manager = concept_manager.lock().unwrap();

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
