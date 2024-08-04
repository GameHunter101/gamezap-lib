use gamezap::new_component;

new_component!(
    ComputeMonitorComponent {
        data: [f32; 6],
        pipeline_index: usize
    }
);

impl ComponentSystem for ComputeMonitorComponent {
    fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        _component_map: &mut AllComponents,
        _engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
        _entities: &mut Vec<Entity>,
        _materials: Option<&(Vec<Material>, usize)>,
        compute_pipelines: &[ComputePipeline],
    ) {
        compute_pipelines[self.pipeline_index].run_compute_shader::<u32>(device, queue);
    }
}
