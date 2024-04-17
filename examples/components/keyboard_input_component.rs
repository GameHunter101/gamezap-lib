use std::{
    any::TypeId,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use gamezap::{
    ecs::{
        component::{Component, ComponentId, ComponentSystem},
        components::transform_component::TransformComponent,
        concepts::ConceptManager,
        entity::EntityId,
        scene::{AllComponents, Scene},
    },
    EngineDetails, EngineSystems,
};

use nalgebra as na;
use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
};

#[derive(Debug, Clone)]
pub struct KeyboardInputComponent {
    parent: EntityId,
    id: ComponentId,
}
impl Default for KeyboardInputComponent {
    fn default() -> Self {
        KeyboardInputComponent {
            parent: EntityId::MAX,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
        }
    }
}

impl ComponentSystem for KeyboardInputComponent {
    fn update(
        &mut self,
        _device: Arc<wgpu::Device>,
        _queue: Arc<wgpu::Queue>,
        component_map: &AllComponents,
        engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
    ) {
        let mut concept_manager = concept_manager.lock().unwrap();
        let transform_component =
            Scene::get_component::<TransformComponent>(component_map.get(&self.parent).unwrap());
        let camera_rotation_matrix = match transform_component {
            Some(transform) => transform.create_rotation_matrix(&concept_manager),
            None => na::Matrix4::identity(),
        };

        let position_concept = concept_manager
            .get_concept_mut::<na::Vector3<f32>>(
                (self.parent, TypeId::of::<TransformComponent>(), 0),
                "position".to_string(),
            )
            .unwrap();

        let details = engine_details.lock().unwrap();

        let speed = 10.0 / (details.last_frame_duration.as_micros() as f32);

        let forward_vector = (camera_rotation_matrix
            * na::Vector3::new(0.0, 0.0, 1.0).to_homogeneous())
        .xyz()
        .normalize();

        let left_vector = forward_vector.cross(&-na::Vector3::y_axis()).normalize();

        for scancode in &details.pressed_scancodes {
            match scancode {
                Scancode::W => {
                    *position_concept += forward_vector * speed;
                }
                Scancode::S => {
                    *position_concept -= forward_vector * speed;
                }
                Scancode::A => {
                    *position_concept -= left_vector * speed;
                }
                Scancode::D => {
                    *position_concept += left_vector * speed;
                }
                Scancode::LCtrl => {
                    position_concept.y -= speed;
                }
                Scancode::Space => {
                    position_concept.y += speed;
                }
                _ => {}
            }
        }
    }

    fn on_event(
        &self,
        event: &Event,
        _component_map: &HashMap<EntityId, Vec<Component>>,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
        _engine_details: &EngineDetails,
        engine_systems: &EngineSystems,
    ) {
        let context = &engine_systems.sdl_context;
        if let Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } = event
        {
            let is_cursor_visible = context.mouse().is_cursor_showing();
            context.mouse().set_relative_mouse_mode(is_cursor_visible);
            context.mouse().show_cursor(!is_cursor_visible);
        }
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
