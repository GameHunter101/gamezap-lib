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
        entity::{EntityId, Entity},
        scene::AllComponents,
    },
    EngineDetails, EngineSystems,
};

// use ultraviolet::{Rotor3, Vec3, Bivec3};
use algoe::{bivector::Bivector, rotor::Rotor3, vector::GeometricOperations};
use nalgebra::Vector3;

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
        _component_map: &mut AllComponents,
        engine_details: Rc<Mutex<EngineDetails>>,
        engine_systems: Rc<Mutex<EngineSystems>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        active_camera_id: Option<EntityId>,
        _entities: &mut Vec<Entity>,
    ) {
        let mut concept_manager = concept_manager.lock().unwrap();
        /* let pitch = *concept_manager
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
            .unwrap(); */

        let systems = engine_systems.lock().unwrap();
        let details = engine_details.lock().unwrap();

        let sdl_context = &systems.sdl_context;
        let mouse = sdl_context.mouse();
        let is_hidden = mouse.relative_mouse_mode();

        let speed = (details.last_frame_duration.as_micros() as f32) / 1000000.0;
        // let speed = 100.0 * details.last_frame_duration.as_micros() as f32;
        if is_hidden {
            if let Some(mouse_state) = details.mouse_state.0 {
                let rotation = *concept_manager
                    .get_concept::<Rotor3>(
                        (
                            active_camera_id.unwrap(),
                            TypeId::of::<TransformComponent>(),
                            0,
                        ),
                        "rotation".to_string(),
                    )
                    .unwrap();
                // println!("Rotation pre: {rotation:?}");
                /* concept_manager
                .modify_concept(
                    (
                        active_camera_id.unwrap(),
                        TypeId::of::<TransformComponent>(),
                        0,
                    ),
                    "pitch".to_string(),
                    pitch + mouse_state.x() as f32 * speed,
                )
                .unwrap(); */
                // let new_rotation = Rotor3::from_rotation_xz(mouse_state.x() as f32 * speed);
                // println!("rotation: {}", mouse_state.x() as f32 * speed);
                /* let new_rotation = Rotor3::from_rotation_xz(-mouse_state.x() as f32 * speed)
                .rotated_by(rotation)
                .normalized(); */
                let first_rotation =
                    (Bivector::new(0.0, 0.0, -1.0) * mouse_state.x() as f32 * speed)
                        .exponentiate()
                        * rotation;

                let forward_vec = first_rotation * Vector3::z_axis().xyz();
                let bivec = forward_vec.wedge(&-Vector3::y_axis().xyz());
                // dbg!(bivec);

                let second_rotation =
                    first_rotation * (bivec * -mouse_state.y() as f32 * speed).exponentiate();

                // dbg!(second_rotation);

                // dbg!(new_rotation);
                // rotation.rotate_by(new_rotation);

                /* if ((yaw - std::f32::consts::FRAC_PI_2).abs() <= 0.1 && mouse_state.y() > 0)
                    || ((yaw + std::f32::consts::FRAC_PI_2).abs() <= 0.1 && mouse_state.y() < 0)
                {
                    return;
                } */

                // let vertical_rotation_bivector = Vec3::unit_y().wedge(rotation * Vec3::unit_z());

                /* rotation.rotate_by(Rotor3::from_angle_plane(
                    mouse_state.y() as f32 * speed,
                    vertical_rotation_bivector,
                )); */

                /* concept_manager
                .modify_concept(
                    (
                        active_camera_id.unwrap(),
                        TypeId::of::<TransformComponent>(),
                        0,
                    ),
                    "yaw".to_string(),
                    yaw + mouse_state.y() as f32 * speed,
                )
                .unwrap(); */
                concept_manager
                    .modify_concept(
                        (
                            active_camera_id.unwrap(),
                            TypeId::of::<TransformComponent>(),
                            0,
                        ),
                        "rotation".to_string(),
                        // first_rotation,
                        second_rotation,
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
