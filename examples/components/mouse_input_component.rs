use std::{
    any::TypeId,
    rc::Rc,
    sync::{Arc, Mutex},
};

use gamezap::{
    ecs::{
        component::{ComponentId, ComponentSystem},
        components::transform_component::TransformComponent,
        concepts::ConceptManager,
        entity::EntityId,
        scene::AllComponents,
    },
    EngineDetails, EngineSystems,
};

#[derive(Debug, Clone)]
pub struct MouseInputComponent {
    parent: EntityId,
    id: ComponentId,
}

impl Default for MouseInputComponent {
    fn default() -> Self {
        MouseInputComponent {
            parent: EntityId::MAX,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
        }
    }
}

impl ComponentSystem for MouseInputComponent {
    fn update(
        &mut self,
        _device: Arc<wgpu::Device>,
        _queue: Arc<wgpu::Queue>,
        _component_map: &AllComponents,
        engine_details: Rc<Mutex<EngineDetails>>,
        engine_systems: Rc<Mutex<EngineSystems>>,
        concept_manager: Arc<Mutex<ConceptManager>>,
        active_camera_id: Option<EntityId>,
    ) {
        let mut concept_manager = concept_manager.lock().unwrap();
        let pitch = *concept_manager
            .get_concept::<f32>(
                (
                    active_camera_id.unwrap(),
                    TypeId::of::<TransformComponent>(),
                    0,
                ),
                "pitch".to_string(),
            )
            .unwrap();

        let yaw = *concept_manager
            .get_concept::<f32>(
                (
                    active_camera_id.unwrap(),
                    TypeId::of::<TransformComponent>(),
                    0,
                ),
                "yaw".to_string(),
            )
            .unwrap();

        let systems = engine_systems.lock().unwrap();
        let details = engine_details.lock().unwrap();

        let sdl_context = &systems.sdl_context;
        let mouse = sdl_context.mouse();
        let is_hidden = mouse.relative_mouse_mode();

        let speed = 5.0 / (details.last_frame_duration.as_micros() as f32);
        if is_hidden {
            if let Some(mouse_state) = details.mouse_state.0 {
                concept_manager
                    .modify_concept(
                        (
                            active_camera_id.unwrap(),
                            TypeId::of::<TransformComponent>(),
                            0,
                        ),
                        "pitch".to_string(),
                        pitch + mouse_state.x() as f32 * speed,
                    )
                    .unwrap();

                if ((yaw - std::f32::consts::FRAC_PI_2).abs() <= 0.1 && mouse_state.y() > 0)
                    || ((yaw + std::f32::consts::FRAC_PI_2).abs() <= 0.1 && mouse_state.y() < 0)
                {
                    return;
                }
                concept_manager
                    .modify_concept(
                        (
                            active_camera_id.unwrap(),
                            TypeId::of::<TransformComponent>(),
                            0,
                        ),
                        "yaw".to_string(),
                        yaw + mouse_state.y() as f32 * speed,
                    )
                    .unwrap();
            }
        }
    }

    fn get_parent_entity(&self) -> EntityId {
        self.parent
    }

    fn get_id(&self) -> ComponentId {
        self.id
    }

    fn update_metadata(&mut self, parent: EntityId, same_component_count: u32) {
        self.parent = parent;
        self.id.0 = parent;
        self.id.2 = same_component_count;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
