use gamezap::new_component;

new_component!(ComputeMonitorComponent {
    pipeline_index: usize
});

impl ComputeMonitorComponent {
    pub fn new(pipeline_index: usize) -> ComputeMonitorComponent {
        ComputeMonitorComponent {
            pipeline_index,
            parent: 0,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
        }
    }
}

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
        _materials: Option<&mut (Vec<Material>, usize)>,
        compute_pipelines: &[ComputePipeline],
    ) {
        match compute_pipelines[self.pipeline_index].run_compute_shader::<f32>(device, queue) {
            Ok(res) => println!("Compute result: {:?}", res),
            Err(err) => println!("ERROR: {:?}", err),
        };
    }
}
