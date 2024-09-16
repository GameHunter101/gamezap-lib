use algoe::bivector::Bivector;
use gamezap::{new_component, ecs::components::transform_component::TransformComponent};
use nalgebra::Vector3;

new_component!(TransparencyComponent {});

impl Default for TransparencyComponent {
    fn default() -> Self {
        TransparencyComponent {
            parent: EntityId::MAX,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
        }
    }
}

impl ComponentSystem for TransparencyComponent {
    fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        component_map: &mut AllComponents,
        engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
        _entities: &mut Vec<Entity>,
        materials: Option<&mut (Vec<Material>, usize)>,
        compute_pipelines: &mut [ComputePipeline],
    ) {
        let details = engine_details.lock().unwrap();
        let time = details.time_elapsed.as_secs_f32();

        let materials = materials.unwrap();
        let selected_material = &mut materials.0[materials.1];
        if let Some((_, buffer)) = &selected_material.uniform_buffer_bind_group() {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[time % 2.0]));
        }

        let output_data = &compute_pipelines[0].pipeline_assets[1];
        if let gamezap::compute::ComputePackagedData::Texture(texture) = output_data {
            selected_material.update_textures(device, &[(texture.clone(), 0)]);
        }

        for comp in component_map.get_mut(&self.parent).unwrap() {
            if let Some(transform) = comp.as_any_mut().downcast_mut::<TransformComponent>() {
                transform.apply_translation(concept_manager.clone(), Vector3::new(0.0, -5.0, 0.0));
                transform.apply_rotation(concept_manager.clone(), (Bivector::new(0.0, 1.0, 0.0) * 0.001).exponentiate());
                transform.apply_translation(concept_manager, Vector3::new(0.0, 5.0, 0.0));
                break;
            }
        }

    }
}
