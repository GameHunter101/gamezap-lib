use gamezap::{compute::ComputePackagedData, new_component, texture::Texture};

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
        compute_pipelines: &mut [ComputePipeline],
    ) {
        match compute_pipelines[self.pipeline_index].grab_array_data::<f32>(device.clone(), 2) {
            Ok(res) => {},// println!("Compute result: {:?}", res),
            Err(err) => println!("ERROR: {:?}", err),
        };

        /* let rgba = image::RgbaImage::from_fn(200, 200, |_, _| image::Rgba([10; 4]));

        compute_pipelines[self.pipeline_index].update_pipeline_assets(
            device.clone(),
            vec![(
                ComputePackagedData::Texture(Rc::new(
                    Texture::from_rgba(&device, &queue, &rgba, None, true, true).unwrap(),
                )),
                0,
            )],
        ) */
    }
}
