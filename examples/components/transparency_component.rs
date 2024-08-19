use gamezap::new_component;

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
    fn update<'a:'b, 'b>(
        &'a mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        _component_map: &'a mut AllComponents,
        engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
        _entities: &'a mut Vec<Entity>,
        materials: Option<&'b mut (Vec<Material>, usize)>,
        compute_pipelines: &'a mut [ComputePipeline],
    ) {
        let details = engine_details.lock().unwrap();
        let time = details.time_elapsed.as_secs_f32();

        let materials = materials.unwrap();
        let selected_material = &mut materials.0[materials.1];
        if let Some((_, buffer)) = &selected_material.uniform_buffer_bind_group() {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[time % 2.0]));
        }

        /* let output_data = &compute_pipelines[0].output_data;
        if let ComputePackagedData::Texture(texture) = output_data {
            selected_material.update_textures(device, vec![texture]);
        } */

    }
}
