#![allow(unused_imports)]

use na::Vector3;
use nalgebra as na;
use std::time::{Duration, Instant};
// use ultraviolet::{Rotor3, Bivec3};
use algoe::{bivector::Bivector, rotor::Rotor3};

use crate::{ecs::scene::{Scene, TextParams}, new_component, ui_manager::UiManager};

use super::transform_component::TransformComponent;

new_component!(
    PhysicsComponent {
        concept_ids: Vec<String>,
        impulses: Vec<Impulse>
    }
);

impl PhysicsComponent {
    pub fn new(
        concept_manager: Rc<Mutex<ConceptManager>>,
        velocity: Vector3<f32>,
        net_force: Vector3<f32>,
        mass: f32,
        angular_velocity: Bivector,
        net_torque: Bivector,
    ) -> Self {
        let mut component = PhysicsComponent {
            parent: EntityId::MAX,
            concept_ids: Vec::new(),
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            impulses: Vec::new(),
        };

        let mut concepts: HashMap<String, Box<dyn Any>> = HashMap::new();

        concepts.insert("velocity".to_string(), Box::new(velocity));
        concepts.insert("net_force".to_string(), Box::new(net_force));
        concepts.insert("mass".to_string(), Box::new(mass));
        concepts.insert("angular_velocity".to_string(), Box::new(angular_velocity));
        concepts.insert("net_torque".to_string(), Box::new(net_torque));

        component.register_component(concept_manager, concepts);

        component
    }

    pub fn add_constant_force(
        &self,
        concept_manager: Rc<Mutex<ConceptManager>>,
        force: Vector3<f32>,
    ) {
        let mut concept_manager = concept_manager.lock().unwrap();
        let net_force = concept_manager
            .get_concept_mut::<Vector3<f32>>(self.id, "net_force".to_string())
            .unwrap();
        *net_force += force;
    }

    pub fn add_impulse(&mut self, force: Vector3<f32>, duration: Duration) {
        self.impulses.push(Impulse {
            force,
            initialized_instant: Instant::now(),
            duration,
        });
    }

    fn sum_impulses(&self) -> Vector3<f32> {
        let impulses = self
            .impulses
            .iter()
            .map(
                |Impulse {
                     force,
                     initialized_instant: _,
                     duration: _,
                 }| force,
            )
            .collect::<Vec<_>>();
        impulses.into_iter().sum()
    }

    fn remove_impulses(&mut self) {
        self.impulses.retain(
            |Impulse {
                 force: _,
                 initialized_instant,
                 duration,
             }| {
                let expiration_time = *initialized_instant + *duration;
                Instant::now() >= expiration_time
            },
        );
    }
}

impl ComponentSystem for PhysicsComponent {
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
        _device: Arc<wgpu::Device>,
        _queue: Arc<wgpu::Queue>,
        component_map: &AllComponents,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: Option<Rc<Mutex<EngineDetails>>>,
        _engine_systems: Option<Rc<Mutex<EngineSystems>>>,
        _ui_manager: Rc<Mutex<UiManager>>,
        _text_items: &mut Vec<TextParams>,
    ) {
        let _transform_component =
            Scene::get_component::<TransformComponent>(&component_map[&self.parent])
                .expect("Physics component expects a transform component on this entity");
    }

    fn update(
        &mut self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &mut AllComponents,
        engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
        _entities: &mut Vec<Entity>,
        _materials: Option<&mut (Vec<Material>, usize)>,
        _compute_pipelines: &mut [ComputePipeline],
        _text_items: &mut Vec<TextParams>,
    ) {
        let mut concept_manager = concept_manager.lock().unwrap();
        let engine_details = engine_details.lock().unwrap();
        let delta_time = (engine_details.last_frame_duration.as_micros() as f32) / 1000.0;

        // First part of linear velocity
        let velocity = concept_manager
            .get_concept::<Vector3<f32>>(self.id, "velocity".to_string())
            .unwrap()
            .clone_owned();

        let angular_velocity = *concept_manager
            .get_concept::<Bivector>(self.id, "angular_velocity".to_string())
            .unwrap();

        let position = concept_manager
            .get_concept_mut::<Vector3<f32>>(
                (self.parent, TypeId::of::<TransformComponent>(), 0),
                "position".to_string(),
            )
            .unwrap();

        *position += velocity * delta_time / 2.0;

        // First part of angular velocity
        // let corrected_angular_velocity = match angular_velocity.to_normalized().mag().is_nan() {
        //     true => (0.0, Bivector::zero()),
        //     false => (angular_velocity.mag(), angular_velocity.to_normalized()),
        // };
        let rotor = Rotor3 {
            scalar: angular_velocity.magnitude().cos(),
            bivector: angular_velocity.to_normalized() * angular_velocity.magnitude().sin(),
        };
        // angular_velocity.scale_by(delta_time / 2.0);
        let rotated_position_slice = rotor * *position;
        *position = rotated_position_slice;

        // Calculating new linear velocity
        let mass = *concept_manager
            .get_concept::<f32>(self.id, "mass".to_string())
            .unwrap();

        let net_force = concept_manager
            .get_concept::<Vector3<f32>>(self.id, "net_force".to_string())
            .unwrap()
            .clone_owned()
            + self.sum_impulses();

        let acceleration = net_force / mass;

        let velocity = concept_manager
            .get_concept_mut::<Vector3<f32>>(self.id, "velocity".to_string())
            .unwrap();

        let new_velocity = velocity.clone_owned() + acceleration * delta_time;
        *velocity = new_velocity;

        // Calculating new angular velocity
        /* let net_torque = *concept_manager
        .get_concept::<Rotor3>(self.id, "net_torque".to_string())
        .unwrap(); */

        // Second part of linear velocity
        let position = concept_manager
            .get_concept_mut::<Vector3<f32>>(
                (self.parent, TypeId::of::<TransformComponent>(), 0),
                "position".to_string(),
            )
            .unwrap();

        *position += new_velocity * delta_time / 2.0;

        self.remove_impulses();
    }
}

#[derive(Debug, Clone)]
pub struct Impulse {
    force: Vector3<f32>,
    initialized_instant: Instant,
    duration: Duration,
}
