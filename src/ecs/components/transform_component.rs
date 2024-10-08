use std::{fmt::Debug, sync::MutexGuard};

use na::{Matrix3, Matrix4, Vector3, Vector4};
// use ultraviolet::{Rotor3, Vec3};
use algoe::rotor::Rotor3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, RenderPass,
};

use nalgebra as na;

use crate::{ecs::{component::Component, scene::TextParams}, model::VertexData, new_component, ui_manager::UiManager};

new_component!(
    TransformComponent {
        concept_ids: Vec<String>,
        buf: Arc<Option<Buffer>>
    }
);

impl TransformComponent {
    pub fn create_rotation_matrix(
        &self,
        concept_manager: &MutexGuard<ConceptManager>,
    ) -> na::Matrix4<f32> {
        let rotation = *concept_manager
            .get_concept::<Rotor3>(self.id, "rotation".to_string())
            .unwrap();

        let rotated_x = (rotation * Vector3::x_axis().xyz()).to_homogeneous();
        let rotated_y = (rotation * Vector3::y_axis().xyz()).to_homogeneous();
        let rotated_z = (rotation * Vector3::z_axis().xyz()).to_homogeneous();

        Matrix4::from_columns(&[
            rotated_x,
            rotated_y,
            rotated_z,
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ])
    }
}

impl VertexData for TransformComponent {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

impl TransformComponent {
    pub fn new(
        concept_manager: Rc<Mutex<ConceptManager>>,
        position: na::Vector3<f32>,
        rotation: Rotor3,
        scale: na::Vector3<f32>,
    ) -> TransformComponent {
        let mut component = TransformComponent {
            parent: EntityId::MAX,
            concept_ids: Vec::new(),
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            buf: Arc::new(None),
        };

        let rotated_x = (rotation * Vector3::x_axis().xyz()).to_homogeneous();
        let rotated_y = (rotation * Vector3::y_axis().xyz()).to_homogeneous();
        let rotated_z = (rotation * Vector3::z_axis().xyz()).to_homogeneous();

        let rotation_matrix = Matrix4::from_columns(&[
            rotated_x,
            rotated_y,
            rotated_z,
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ]);

        let translation_matrix = na::Translation3::from(position).to_homogeneous();
        let scale_matrix = na::Scale3::from(scale).to_homogeneous();
        let transform_matrix = scale_matrix * rotation_matrix * translation_matrix;

        let mut concepts: HashMap<String, Box<dyn Any>> = HashMap::new();

        concepts.insert("matrix".to_string(), Box::new(transform_matrix));
        concepts.insert("position".to_string(), Box::new(position));
        concepts.insert("rotation".to_string(), Box::new(rotation));
        concepts.insert("scale".to_string(), Box::new(scale));

        component.register_component(concept_manager, concepts);
        component
    }

    pub fn default(concept_manager: Rc<Mutex<ConceptManager>>) -> Self {
        let mut component = Self {
            parent: EntityId::MAX,
            concept_ids: Vec::new(),
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            buf: Arc::new(None),
        };

        let mut concepts: HashMap<String, Box<dyn Any>> = HashMap::new();

        concepts.insert(
            "matrix".to_string(),
            Box::new(na::Matrix4::<f32>::identity()),
        );
        concepts.insert(
            "position".to_string(),
            Box::new(na::Vector3::<f32>::zeros()),
        );

        concepts.insert("rotation".to_string(), Box::<Rotor3>::default());
        concepts.insert(
            "scale".to_string(),
            Box::new(na::Vector3::new(1.0, 1.0, 1.0)),
        );

        component.register_component(concept_manager, concepts);

        component
    }

    pub fn apply_translation(
        &self,
        concept_manager: Rc<Mutex<ConceptManager>>,
        translation: Vector3<f32>,
    ) {
        let mut concept_manager = concept_manager.lock().unwrap();
        let current_position = concept_manager
            .get_concept_mut::<Vector3<f32>>(self.id, String::from("position"))
            .unwrap();
        *current_position += translation;

        let new_matrix = na::Translation3::from(translation);

        let transform = concept_manager
            .get_concept_mut::<Matrix4<f32>>(self.id, String::from("matrix"))
            .unwrap();

        *transform *= new_matrix.to_homogeneous();
    }

    pub fn apply_rotation(&self, concept_manager: Rc<Mutex<ConceptManager>>, rotation: Rotor3) {
        let mut concept_manager = concept_manager.lock().unwrap();
        let current_rotation = concept_manager
            .get_concept_mut::<Rotor3>(self.id, String::from("rotation"))
            .unwrap();
        *current_rotation = *current_rotation * rotation;

        let new_matrix = Matrix3::from_columns(&[
            rotation * Vector3::x_axis().xyz(),
            rotation * Vector3::y_axis().xyz(),
            rotation * Vector3::z_axis().xyz(),
        ]);

        let transform = concept_manager
            .get_concept_mut::<Matrix4<f32>>(self.id, String::from("matrix"))
            .unwrap();

        *transform *= new_matrix.to_homogeneous();
    }

    pub fn apply_scale(&self, concept_manager: Rc<Mutex<ConceptManager>>, dilation: Vector3<f32>) {
        let mut concept_manager = concept_manager.lock().unwrap();
        let current_scale = concept_manager
            .get_concept_mut::<Vector3<f32>>(self.id, String::from("scale"))
            .unwrap();
        *current_scale += dilation;

        let new_matrix = Matrix4::new_nonuniform_scaling(&dilation);
        let transform = concept_manager
            .get_concept_mut::<Matrix4<f32>>(self.id, String::from("matrix"))
            .unwrap();

        *transform *= new_matrix;
    }

    pub fn update_buffer(
        &mut self,
        concept_manager: Rc<Mutex<ConceptManager>>,
        device: Arc<Device>,
    ) {
        let concept_manager = concept_manager.lock().unwrap();
        /* let position = concept_manager
            .get_concept::<na::Vector3<f32>>(self.id, "position".to_string())
            .unwrap();

        let scale = concept_manager
            .get_concept::<na::Vector3<f32>>(self.id, "scale".to_string())
            .unwrap();

        let rot_matrix = self.create_rotation_matrix(&concept_manager);

        let matrix = na::Matrix4::<f32>::new_translation(position)
            * rot_matrix
            * na::Matrix4::<f32>::new_nonuniform_scaling(scale); */
        let matrix = *concept_manager
            .get_concept::<Matrix4<f32>>(self.id, "matrix".to_string())
            .unwrap();
        let matrix_as_arr: [[f32; 4]; 4] = matrix.into();

        let new_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Entity Transform Buffer"),
            contents: bytemuck::cast_slice(&matrix_as_arr),
            usage: BufferUsages::VERTEX,
        });
        self.buf = Arc::new(Some(new_buffer));
    }
}

impl ComponentSystem for TransformComponent {
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
        device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &AllComponents,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: Option<Rc<Mutex<EngineDetails>>>,
        _engine_systems: Option<Rc<Mutex<EngineSystems>>>,
        _ui_manager: Rc<Mutex<UiManager>>,
        _text_items: &mut Vec<TextParams>,
    ) {
        let concept_manager = concept_manager.lock().unwrap();
        let matrix = concept_manager
            .get_concept::<na::Matrix4<f32>>(self.id, "matrix".to_string())
            .unwrap();
        let matrix_as_arr: [[f32; 4]; 4] = matrix.clone_owned().into();
        self.buf = Arc::new(Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Transform Component Buffer"),
            contents: bytemuck::cast_slice(&matrix_as_arr),
            usage: BufferUsages::VERTEX,
        })));
    }

    fn update(
        &mut self,
        device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &mut AllComponents,
        _engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
        _entities: &mut Vec<Entity>,
        _materials: Option<&mut (Vec<Material>, usize)>,
        _compute_pipelines: &mut [ComputePipeline],
        _text_items: &mut Vec<TextParams>,
    ) {
        self.update_buffer(concept_manager, device);
    }

    fn render<'a: 'b, 'b>(
        &'a self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        render_pass: &mut RenderPass<'b>,
        _component_map: &'a HashMap<EntityId, Vec<Component>>,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: &EngineDetails,
        _engine_systems: &EngineSystems,
    ) {
        if let Some(buf) = self.buf.as_ref() {
            render_pass.set_vertex_buffer(1, buf.slice(..));
        }
    }
}
