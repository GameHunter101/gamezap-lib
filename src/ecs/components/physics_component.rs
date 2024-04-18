use std::{
    any::{Any, TypeId},
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use na::Vector3;
use nalgebra as na;

use crate::{
    ecs::{
        component::{ComponentId, ComponentSystem},
        concepts::ConceptManager,
        entity::EntityId,
        scene::{AllComponents, Scene},
    },
    EngineDetails, EngineSystems,
};

use super::transform_component::TransformComponent;

#[derive(Debug, Clone)]
pub struct PhysicsComponent {
    parent: EntityId,
    concept_ids: Vec<String>,
    id: ComponentId,
}

impl PhysicsComponent {
    pub fn new(
        concept_manager: Arc<Mutex<ConceptManager>>,
        velocity: Vector3<f32>,
        net_force: Vector3<f32>,
        mass: f32,
    ) -> Self {
        let mut component = PhysicsComponent {
            parent: EntityId::MAX,
            concept_ids: Vec::new(),
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
        };

        let mut concepts: HashMap<String, Box<dyn Any>> = HashMap::new();

        concepts.insert("velocity".to_string(), Box::new(velocity));
        concepts.insert("net_force".to_string(), Box::new(net_force));
        concepts.insert("mass".to_string(), Box::new(mass));

        component.register_component(concept_manager, concepts);

        component
    }

    pub fn add_constant_force(
        &self,
        concept_manager: Arc<Mutex<ConceptManager>>,
        force: Vector3<f32>,
    ) {
        let mut concept_manager = concept_manager.lock().unwrap();
        let net_force = concept_manager
            .get_concept_mut::<Vector3<f32>>(self.id, "net_force".to_string())
            .unwrap();
        *net_force += force;
    }
}

impl ComponentSystem for PhysicsComponent {
    fn register_component(
        &mut self,
        concept_manager: Arc<Mutex<ConceptManager>>,
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
        _device: Arc<wgpu::Device>,
        _queue: Arc<wgpu::Queue>,
        component_map: &AllComponents,
        _concept_manager: Arc<Mutex<ConceptManager>>,
        _engine_details: Option<Rc<Mutex<EngineDetails>>>,
        _engine_systems: Option<Rc<Mutex<EngineSystems>>>,
    ) {
        let _transform_component =
            Scene::get_component::<TransformComponent>(&component_map[&self.parent])
                .expect("Physics component expects a transform component on this entity");
    }

    fn update(
        &mut self,
        _device: Arc<wgpu::Device>,
        _queue: Arc<wgpu::Queue>,
        _component_map: &AllComponents,
        engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
        concept_manager: Arc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
    ) {
        let mut concept_manager = concept_manager.lock().unwrap();
        let engine_details = engine_details.lock().unwrap();
        let delta_time = (engine_details.last_frame_duration.as_micros() as f32) / 1000.0;

        let velocity = concept_manager
            .get_concept::<Vector3<f32>>(self.id, "velocity".to_string())
            .unwrap()
            .clone_owned();

        let position = concept_manager
            .get_concept_mut::<Vector3<f32>>(
                (self.parent, TypeId::of::<TransformComponent>(), 0),
                "position".to_string(),
            )
            .unwrap();

        *position += velocity * delta_time / 2.0;

        let mass = *concept_manager
            .get_concept::<f32>(self.id, "mass".to_string())
            .unwrap();

        let acceleration = concept_manager
            .get_concept::<Vector3<f32>>(self.id, "net_force".to_string())
            .unwrap()
            .clone_owned()
            / mass;

        let velocity = concept_manager
            .get_concept_mut::<Vector3<f32>>(self.id, "velocity".to_string())
            .unwrap();

        let new_velocity = velocity.clone_owned() + acceleration * delta_time;
        *velocity = new_velocity;

        let position = concept_manager
            .get_concept_mut::<Vector3<f32>>(
                (self.parent, TypeId::of::<TransformComponent>(), 0),
                "position".to_string(),
            )
            .unwrap();

        *position += new_velocity * delta_time / 2.0;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
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
