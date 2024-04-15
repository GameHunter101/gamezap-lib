use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex, MutexGuard}, rc::Rc,
};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, Queue, RenderPass,
};

use nalgebra as na;

use crate::{
    ecs::component::{Component, ComponentId, ComponentSystem},
    model::VertexData,
    EngineSystems, EngineDetails,
};

use super::super::{concepts::ConceptManager, entity::EntityId, scene::AllComponents};

#[derive(Debug, Clone)]
pub struct TransformComponent {
    parent: EntityId,
    concept_ids: Vec<String>,
    id: ComponentId,
    buf: Arc<Option<Buffer>>,
}

impl TransformComponent {
    pub fn create_rotation_matrix(
        &self,
        concept_manager: &MutexGuard<ConceptManager>,
    ) -> na::Matrix4<f32> {
        let yaw = concept_manager
            .get_concept::<f32>(self.id, "yaw".to_string())
            .unwrap();

        let pitch = concept_manager
            .get_concept::<f32>(self.id, "pitch".to_string())
            .unwrap();

        let roll = concept_manager
            .get_concept::<f32>(self.id, "roll".to_string())
            .unwrap();

        #[rustfmt::skip]
        let rotation_matrix = na::Matrix3::new(
            pitch.cos() * roll.cos(), yaw.sin() * pitch.sin() * roll.cos() - yaw.cos() * roll.sin(), yaw.cos() * pitch.sin() * roll.cos() + yaw.sin() * roll.sin(), 
            pitch.cos() * roll.sin(), yaw.sin() * pitch.sin() * roll.sin() + yaw.cos() * roll.cos(), yaw.cos() * pitch.sin() * roll.sin() - yaw.sin() * roll.cos(),
            -1.0 * pitch.sin(), yaw.sin() * pitch.cos(), yaw.cos() * pitch.cos()
        ).to_homogeneous();
        rotation_matrix
    }
}

impl VertexData for TransformComponent {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

impl TransformComponent {
    pub fn new(
        concept_manager: Rc<Mutex<ConceptManager>>,
        position: na::Vector3<f32>,
        roll: f32,
        pitch: f32,
        yaw: f32,
        scale: na::Vector3<f32>,
    ) -> TransformComponent {
        let mut component = TransformComponent {
            parent: EntityId::MAX,
            concept_ids: Vec::new(),
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            buf: Arc::new(None),
        };

        #[rustfmt::skip]
        let rotation_matrix = na::Matrix3::new(
            yaw.cos() * pitch.cos(), yaw.cos() * pitch.sin() * roll.sin() - yaw.sin() * roll.cos(), yaw.cos() * pitch.sin() * roll.cos() + yaw.sin() * roll.sin(),
            yaw.sin() * pitch.cos(), yaw.sin() * pitch.sin() * roll.sin() + yaw.cos() * roll.cos(), yaw.sin() * pitch.sin() * roll.cos() - yaw.cos() * roll.sin(),
            -1.0 * pitch.sin(), pitch.cos() * roll.sin(), pitch.cos() * roll.cos()
        ).to_homogeneous();
        let translation_matrix = na::Translation3::from(position).to_homogeneous();
        let scale_matrix = na::Scale3::from(scale).to_homogeneous();
        let transform_matrix = scale_matrix * rotation_matrix * translation_matrix;


        let mut concepts: HashMap<String, Box<dyn Any>> = HashMap::new();

        concepts.insert("matrix".to_string(), Box::new(transform_matrix));
        concepts.insert("position".to_string(), Box::new(position));
        concepts.insert("roll".to_string(), Box::new(roll));
        concepts.insert("pitch".to_string(), Box::new(pitch));
        concepts.insert("yaw".to_string(), Box::new(yaw));
        concepts.insert("scale".to_string(), Box::new(scale));

        component.register_component(concept_manager, concepts);
        component
    }

    pub fn default(concept_manager: Rc<Mutex<ConceptManager>>) -> Self {
        let mut component = Self {
            parent: EntityId::MAX,
            concept_ids: Vec::new(),
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            buf: Arc::new(None),
        };

        let mut concepts: HashMap<String, Box<dyn Any>> = HashMap::new();

        concepts.insert(
            "matrix".to_string(),
            Box::new(na::Matrix4::<f32>::identity()),
        );
        concepts.insert(
            "position".to_string(),
            Box::new(na::Vector3::<f32>::zeros()),
        );
        concepts.insert("roll".to_string(), Box::new(0.0));
        concepts.insert("pitch".to_string(), Box::new(0.0));
        concepts.insert("yaw".to_string(), Box::new(0.0));
        concepts.insert(
            "scale".to_string(),
            Box::new(na::Vector3::new(1.0, 1.0, 1.0)),
        );

        component.register_component(concept_manager, concepts);

        component
    }

    pub fn update_buffer(
        &mut self,
        concept_manager: Rc<Mutex<ConceptManager>>,
        device: Arc<Device>,
    ) {
        let concept_manager = concept_manager.lock().unwrap();
        let position = concept_manager
            .get_concept::<na::Vector3<f32>>(self.id, "position".to_string())
            .unwrap();
        let matrix = self.create_rotation_matrix(&concept_manager) * na::Matrix4::<f32>::new_translation(position);
        let matrix_as_arr: [[f32; 4]; 4] = matrix.into();

        let new_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Entity Transform Buffer"),
            contents: bytemuck::cast_slice(&matrix_as_arr),
            usage: BufferUsages::VERTEX,
        });
        self.buf = Arc::new(Some(new_buffer));
    }
}

impl ComponentSystem for TransformComponent {
    fn register_component(
        &mut self,
        concept_manager: Rc<Mutex<ConceptManager>>,
        data: HashMap<String, Box<dyn Any>>,
    ) {
        self.concept_ids = data.keys().cloned().collect();

        concept_manager.lock().unwrap().register_component_concepts(self.id, data);
    }

    fn initialize(
        &mut self,
        device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &AllComponents,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: Option<Rc<Mutex<EngineDetails>>>,
        _engine_systems: Option<Rc<Mutex<EngineSystems>>>,
    ) {
        let concept_manager = concept_manager.lock().unwrap();
        let matrix = concept_manager
            .get_concept::<na::Matrix4<f32>>(self.id, "matrix".to_string())
            .unwrap();
        let matrix_as_arr: [[f32; 4]; 4] = matrix.clone_owned().into();
        self.buf = Arc::new(Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Transform Component Buffer"),
            contents: bytemuck::cast_slice(&matrix_as_arr),
            usage: BufferUsages::VERTEX,
        })));
    }

    fn update(
        &mut self,
        device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &AllComponents,
        _engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
    ) {
        self.update_buffer(concept_manager, device);
    }

    fn render<'a: 'b, 'b>(
        &'a self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        render_pass: &mut RenderPass<'b>,
        _component_map: &'a HashMap<EntityId, Vec<Component>>,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: &EngineDetails,
        _engine_systems: &EngineSystems,
    ) {
        if let Some(buf) = self.buf.as_ref() {
            render_pass.set_vertex_buffer(1, buf.slice(..));
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
