#![allow(unused_imports)]
use std::fmt::Debug;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, RenderPass,
};

use crate::{
    model::Vertex,
    new_component,
    ui_manager::UiManager, ecs::scene::TextParams,
};

#[derive(Debug)]
pub enum MeshComponentError {
    FailedToLoadObj,
    FailedToLoadMtl,
}

new_component!(MeshComponent {
    concept_ids: Vec<String>,
    mesh_count: usize,
    vertex_buffers: Arc<[Option<Buffer>]>,
    index_buffers: Arc<[Option<Buffer>]>
}, render_order: usize::MAX);

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
            mesh_count: 1,
            vertex_buffers: Arc::from(vec![None].into_boxed_slice()),
            index_buffers: Arc::from(vec![None].into_boxed_slice()),
        };

        let mut concepts: HashMap<String, Box<dyn Any>> = HashMap::new();
        concepts.insert("vertices".to_string(), Box::new(vec![vertices]));
        concepts.insert("indices".to_string(), Box::new(vec![indices]));

        component.register_component(concept_manager, concepts);

        component
    }

    pub fn from_obj(
        concept_manager: Rc<Mutex<ConceptManager>>,
        obj_path: &str,
        expect_material: bool,
    ) -> Result<Self, MeshComponentError> {
        let path = std::path::Path::new(&std::env::current_dir().unwrap()).join(obj_path);
        let obj_load_res = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        );

        if let Ok((models, materials_res)) = obj_load_res {
            if materials_res.is_err() && expect_material {
                return Err(MeshComponentError::FailedToLoadMtl);
            }

            // let materials = materials_res.unwrap_or(vec![tobj::Material::default()]);

            let meshes = models.into_iter().map(|m| {
                let vertices = (0..m.mesh.positions.len() / 3)
                    .map(|i| Vertex {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        normal: [
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2],
                        ],
                        tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                    })
                    .collect::<Vec<_>>();

                (vertices, m.mesh.indices)
            });

            let (vertices, indices): (Vec<_>, Vec<_>) = meshes.unzip();

            let mut component = MeshComponent {
                parent: EntityId::MAX,
                concept_ids: Vec::new(),
                id: (EntityId::MAX, TypeId::of::<Self>(), 0),
                mesh_count: vertices.len(),
                vertex_buffers: Arc::from(vec![None].into_boxed_slice()),
                index_buffers: Arc::from(vec![None].into_boxed_slice()),
            };

            let mut concepts: HashMap<String, Box<dyn Any>> = HashMap::new();
            concepts.insert("vertices".to_string(), Box::new(vertices));
            concepts.insert("indices".to_string(), Box::new(indices));

            component.register_component(concept_manager, concepts);

            return Ok(component);
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
        _engine_details: Option<Rc<Mutex<EngineDetails>>>,
        _engine_systems: Option<Rc<Mutex<EngineSystems>>>,
        _ui_manager: Rc<Mutex<UiManager>>,
        _text_items: &mut Vec<TextParams>,
    ) {
        let concept_manager = concept_manager.lock().unwrap();
        let vertices = concept_manager
            .get_concept::<Vec<Vec<Vertex>>>(self.id, "vertices".to_string())
            .unwrap();

        let indices = concept_manager
            .get_concept::<Vec<Vec<u32>>>(self.id, "indices".to_string())
            .unwrap();

        let buffers = (0..self.mesh_count).map(|i| {
            let current_vertices = &vertices[i];
            let current_indices = &indices[i];

            let vert_buf = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Entity Vertex Buffer"),
                contents: bytemuck::cast_slice(current_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let ind_buf = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Entity Index Buffer"),
                contents: bytemuck::cast_slice(current_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            (Some(vert_buf), Some(ind_buf))
        });

        let (vert_bufs, ind_bufs): (Vec<_>, Vec<_>) = buffers.unzip();
        self.vertex_buffers = Arc::from(vert_bufs.into_boxed_slice());
        self.index_buffers = Arc::from(ind_bufs.into_boxed_slice());
    }

    fn render<'a: 'b, 'b>(
        &'a self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        render_pass: &mut RenderPass<'b>,
        _component_map: &'a AllComponents,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: &EngineDetails,
        _engine_systems: &EngineSystems,
    ) {
        let concept_manager = concept_manager.lock().unwrap();

        for i in 0..self.mesh_count {
            let vertex_buffer_opt = self.vertex_buffers[i].as_ref();
            if let Some(vertex_buffer) = &vertex_buffer_opt {
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            }

            let index_buffer_opt = self.index_buffers[i].as_ref();
            let indices = &concept_manager
                .get_concept::<Vec<Vec<u32>>>(self.id, "indices".to_string())
                .unwrap()[i];

            if let Some(index_buffer) = index_buffer_opt {
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            }
        }
    }
}
