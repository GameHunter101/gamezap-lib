use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex}, rc::Rc,
};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindingType, Buffer, BufferBindingType, BufferUsages, Device, Queue, ShaderStages,
};

use nalgebra as na;

use crate::{
    ecs::{
        component::{ComponentId, ComponentSystem},
        scene::Scene,
    },
    EngineDetails, EngineSystems,
};

use super::{
    super::{concepts::ConceptManager, entity::EntityId, scene::AllComponents},
    transform_component::TransformComponent,
};

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
pub struct RawCameraData {
    pub cam_pos: [f32; 4],
    pub cam_mat: [[f32; 4]; 4],
}

impl Default for RawCameraData {
    fn default() -> Self {
        RawCameraData {
            cam_pos: [0.0; 4],
            cam_mat: na::Matrix3::<f32>::identity().to_homogeneous().into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CameraComponent {
    parent: EntityId,
    concept_ids: Vec<String>,
    id: ComponentId,
    buf: Arc<Option<Buffer>>,
    raw_data: RawCameraData,
}

impl CameraComponent {
    pub fn new_2d(concept_manager: Rc<Mutex<ConceptManager>>, window_size: (u32, u32)) -> Self {
        let mut component = CameraComponent {
            parent: EntityId::MAX,
            concept_ids: Vec::new(),
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            buf: Arc::new(None),
            raw_data: RawCameraData::default(),
        };

        let mut concepts: HashMap<String, Box<dyn Any>> = HashMap::new();

        concepts.insert(
            "view_to_projected_mat".to_string(),
            Box::new(na::Matrix4::<f32>::identity()),
        );
        concepts.insert(
            "aspect_ratio".to_string(),
            Box::new(window_size.0 as f32 / window_size.1 as f32),
        );
        concepts.insert("fov".to_string(), Box::new(0.0_f32));
        concepts.insert("near_plane".to_string(), Box::new(0.0_f32));
        concepts.insert("far_plane".to_string(), Box::new(0.0_f32));

        component.register_component(concept_manager, concepts);
        component
    }

    pub fn new_3d(
        concept_manager: Rc<Mutex<ConceptManager>>,
        window_size: (u32, u32),
        fov: f32,
        near_plane: f32,
        far_plane: f32,
    ) -> Self {
        let aspect_ratio = window_size.0 as f32 / window_size.1 as f32;
        let c = 1.0 / (fov / 2.0).atan();
        #[rustfmt::skip]
        let view_proj = na::Matrix4::new(
            c / aspect_ratio, 0.0, 0.0, 0.0,
            0.0, c, 0.0, 0.0,
            0.0, 0.0, 1.0 * (far_plane + near_plane)/(far_plane - near_plane), -1.0 * (2.0 * far_plane * near_plane) / (far_plane - near_plane),
            0.0, 0.0, 1.0, 0.0
        );
        let mut component = CameraComponent {
            parent: EntityId::MAX,
            concept_ids: Vec::new(),
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            buf: Arc::new(None),
            raw_data: RawCameraData::default(),
        };

        let mut concepts: HashMap<String, Box<dyn Any>> = HashMap::new();

        concepts.insert("view_to_projected_mat".to_string(), Box::new(view_proj));
        concepts.insert(
            "aspect_ratio".to_string(),
            Box::new(window_size.0 as f32 / window_size.1 as f32),
        );
        concepts.insert("fov".to_string(), Box::new(fov));
        concepts.insert("near_plane".to_string(), Box::new(near_plane));
        concepts.insert("far_plane".to_string(), Box::new(far_plane));

        component.register_component(concept_manager, concepts);

        component
    }

    pub fn camera_bind_group_layout(device: Arc<Device>) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Default Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    pub fn create_camera_buffer(&self, device: Arc<Device>) -> Buffer {
        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[self.raw_data]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        camera_buffer
    }

    pub fn create_camera_bind_group(&self, device: Arc<Device>) -> BindGroup {
        let buf_clone = self.buf.clone();
        let buffer = buf_clone.as_ref();
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &Self::camera_bind_group_layout(device.clone()),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_ref().unwrap().as_entire_binding(),
            }],
        });
        bind_group
    }
}

impl ComponentSystem for CameraComponent {
    fn register_component(
        &mut self,
        concept_manager: Rc<Mutex<ConceptManager>>,
        data: HashMap<String, Box<dyn Any>>,
    ) {
        self.concept_ids = data.keys().cloned().collect();

        concept_manager.lock().unwrap().register_component_concepts(self.id, data);
    }

    fn initialize(
        &mut self,
        device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &AllComponents,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: Option<Rc<Mutex<EngineDetails>>>,
        _engine_systems: Option<Rc<Mutex<EngineSystems>>>,
    ) {
        let concept_manager = concept_manager.lock().unwrap();
        let position_concept = concept_manager.get_concept::<na::Vector3<f32>>(
            (self.parent, TypeId::of::<TransformComponent>(), 0),
            "position".to_string(),
        );
        let position = match position_concept {
            Ok(position) => *position,
            Err(_) => na::Vector3::zeros(),
        };
        self.raw_data.cam_pos = position.to_homogeneous().into();
        self.buf = Arc::new(Some(self.create_camera_buffer(device)));
    }

    fn update(
        &mut self,
        _device: Arc<Device>,
        queue: Arc<Queue>,
        component_map: &mut AllComponents,
        engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
    ) {
        let mut concept_manager = concept_manager.lock().unwrap();
        let aspect_ratio = concept_manager
            .get_concept_mut::<f32>(self.id, "aspect_ratio".to_string())
            .unwrap();
        *aspect_ratio = engine_details.lock().unwrap().window_aspect_ratio;

        let position = concept_manager
            .get_concept::<na::Vector3<f32>>(
                (self.parent, TypeId::of::<TransformComponent>(), 0),
                "position".to_string(),
            )
            .unwrap();
        self.raw_data.cam_pos = position.to_homogeneous().into();

        let view_to_projected_mat = concept_manager
            .get_concept::<na::Matrix4<f32>>(self.id, "view_to_projected_mat".to_string())
            .unwrap();
        let transform_component =
            Scene::get_component::<TransformComponent>(component_map.get(&self.parent).unwrap());
        let rotation_matrix = match transform_component {
            Some(transform) => transform.create_rotation_matrix(&concept_manager),
            None => na::Matrix4::identity(),
        };
        // println!("{rotation_matrix}");
        let world_to_view_mat = na::Matrix4::new_translation(position) * rotation_matrix;
        let cam_mat = view_to_projected_mat * world_to_view_mat.try_inverse().unwrap();
        // println!("{cam_mat}");
        self.raw_data.cam_mat = cam_mat.into();
        let buf_clone = self.buf.clone();
        let buffer = buf_clone.as_ref();

        queue.write_buffer(
            buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&[self.raw_data]),
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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
