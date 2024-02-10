#![allow(unused)]
use crate::{
    ecs::entity::Entity,
    model::{Mesh, Vertex, VertexData},
    texture::Texture,
    EngineDetails,
};
use std::{
    any::Any,
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
};

use cool_utils::data_structures::tree::Tree;
use smaa::SmaaTarget;
use wgpu::{
    BindGroup, BindGroupDescriptor, CommandEncoderDescriptor, Device, Queue, RenderPass, Surface,
    TextureFormat,
};

use nalgebra as na;

use crate::pipeline::Pipeline;

use super::{
    component::{
        AsAny, CameraComponent, Component, ComponentSystem, ComponentType, Material, MaterialId,
        TransformComponent,
    },
    entity::{self, EntityId},
};

pub struct Scene {
    entities: Arc<Mutex<Vec<Entity>>>,
    total_entites_created: u32,
    pipelines: HashMap<MaterialId, Pipeline>,
    components: Arc<Mutex<HashMap<EntityId, Vec<Component>>>>,
    materials: Arc<Mutex<HashMap<EntityId, (Vec<Material>, usize)>>>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    color_format: TextureFormat,
    active_camera_id: Option<EntityId>,
}

impl Scene {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, color_format: TextureFormat) -> Self {
        let root_index = vec![0];
        Self {
            entities: Arc::new(Mutex::new(vec![])),
            total_entites_created: 0,
            pipelines: HashMap::new(),
            components: Arc::new(Mutex::new(HashMap::new())),
            materials: Arc::new(Mutex::new(HashMap::new())),
            device,
            queue,
            color_format,
            active_camera_id: None,
        }
    }

    pub fn create_entity(
        &mut self,
        parent: EntityId,
        enabled: bool,
        components: Vec<Component>,
        materials: Option<(Vec<Material>, usize)>,
    ) -> EntityId {
        let new_entity_id = self.total_entites_created;
        let new_entity = Entity::new(new_entity_id, enabled, parent, Vec::new());
        if let Some((materials, active_material_index)) = materials {
            let active_material = &materials[active_material_index];
            let active_material_id = active_material.id().clone();
            if !self.pipelines.contains_key(&active_material_id) {
                let new_pipeline = Pipeline::new(
                    self.device.clone(),
                    self.color_format,
                    &[Vertex::desc(), Mesh::desc()],
                    &active_material_id,
                );
                self.pipelines.insert(active_material_id, new_pipeline);
            }
        }
        self.entities.lock().unwrap().push(new_entity);
        self.total_entites_created += 1;
        new_entity_id
    }

    pub fn update(
        &mut self,
        engine_details: Arc<Mutex<EngineDetails>>,
        smaa_target: Arc<Mutex<SmaaTarget>>,
        surface: Arc<Surface>,
        depth_texture: Arc<Texture>,
    ) {
        let device = self.device.clone();
        let queue = self.queue.clone();

        let entities_arc = self.entities.clone();
        let entities = entities_arc.lock().unwrap();

        let materials_arc = self.materials.clone();
        let materials = self.materials.lock().unwrap();

        let components_arc = self.components.clone();
        let mut components = components_arc.lock().unwrap();

        let mut non_updated_entities: Vec<EntityId> = components.keys().cloned().collect();
        let camera_bind_group = self.create_camera_bind_group();

        let mats_arc: Arc<Mutex<Vec<Material>>> = Arc::new(Mutex::new(Vec::new()));
        let mats = mats_arc.lock().unwrap();

        let output = surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut smaa_binding = smaa_target.lock().unwrap();
        let smaa_frame = smaa_binding.start_frame(&device, &queue, &view);

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Scene Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Scene Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &smaa_frame,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_bind_group(1, &camera_bind_group, &[]);

        for (pipeline_id, pipeline) in &self.pipelines {
            for entity in entities.iter() {
                let entity_materials = materials.get(entity.id());
                if let Some((materials, active_material_index)) = entity_materials {
                    let active_material = &materials[*active_material_index];
                    if active_material.id() == pipeline_id {
                        render_pass.set_bind_group(0, active_material.bind_group(), &[]);
                        for component in components.get_mut(entity.id()).unwrap().iter_mut() {
                            component.update(
                                device.clone(),
                                queue.clone(),
                                components_arc.clone(),
                                engine_details.clone(),
                                &mut render_pass,
                            );
                        }
                        non_updated_entities
                            .iter_mut()
                            .filter(|i| **i != entity.id().clone());
                    }
                }
            }
        }

        for entity_id in &non_updated_entities {
            for component in components.get_mut(entity_id).unwrap() {
                component.update(
                    device.clone(),
                    queue.clone(),
                    components_arc.clone(),
                    engine_details.clone(),
                    &mut render_pass,
                );
            }
        }
    }

    pub fn initialize(&mut self) {
        let device = self.device.clone();
        let queue = self.queue.clone();

        let entities_arc = self.entities.clone();
        let entities = entities_arc.lock().unwrap();
        let components_arc = self.components.clone();
        let mut components = components_arc.lock().unwrap();

        for entity in entities.iter() {
            for component in components.get_mut(entity.id()).unwrap() {
                component.initialize(device.clone(), queue.clone(), components_arc.clone());
            }
        }
    }

    pub fn create_camera_bind_group(&self) -> BindGroup {
        let device = self.device.clone();

        let components_arc = self.components.clone();
        let components = components_arc.lock().unwrap();

        if let Some(active_camera_id) = self.active_camera_id {
            let camera_component = Scene::find_specific_component::<CameraComponent>(&*components[&active_camera_id], ComponentType::Camera);
            let transform_component = Scene::find_specific_component::<TransformComponent>(&*components[&active_camera_id], ComponentType::Transform);
            let position = match transform_component {
                Some(comp) => comp.position().clone(),
                None => na::Vector3::new(0.0, 0.0, 0.0),
            };
            let bind_group = camera_component.unwrap().create_camera_bind_group(device.clone(), position);
            return bind_group;
        }

        let cam = CameraComponent::new(u32::MAX);
        cam.create_camera_bind_group(device.clone(), na::Vector3::new(0.0, 0.0, 0.0))
    }

    pub fn find_specific_component<T: ComponentSystem + Any>(components: &[Component], component_type: ComponentType) -> Option<&T>{
        for component in components {
            if component.component_type() == component_type {
                return component.as_any().downcast_ref::<T>();
            }
        }
        None
    }
}
