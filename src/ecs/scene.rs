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
        AsAny, CameraComponent, Component, ComponentSystem, MaterialComponent, MaterialId,
        TransformComponent,
    },
    entity::{self, EntityId},
};

pub struct Scene {
    entities: Arc<Mutex<Vec<Entity>>>,
    total_entites_created: u32,
    pipelines: Vec<Pipeline>,
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
            pipelines: vec![],
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
        materials: Vec<MaterialComponent>,
    ) {
        let new_entity = Entity::new(
            self.total_entites_created,
            enabled,
            parent,
            vec![],
            components,
            materials,
        );
        if let Some(mat_id) = new_entity.active_material_id() {
            if !self.does_pipeline_exist(&mat_id) {
                let new_pipeline = Pipeline::new(
                    self.device.clone(),
                    self.color_format,
                    &[Vertex::desc(), Mesh::desc()],
                    mat_id,
                );
                self.pipelines.push(new_pipeline);
            }
        }
        self.entities.lock().unwrap().push(new_entity);
    }

    fn does_pipeline_exist(&self, id: &MaterialId) -> bool {
        for pipeline in &self.pipelines {
            if pipeline.id() == id {
                return true;
            }
        }
        return false;
    }

    fn sort_entities_by_index(&self) -> (Vec<usize>, Vec<usize>) {
        let mut pipeline_indices = Vec::with_capacity(self.pipelines.len());
        let mut no_pipeline_indices = Vec::with_capacity(self.entities.lock().unwrap().len());
        let entities_arc = self.entities.clone();
        let entities = entities_arc.lock().unwrap();
        for pipeline in &self.pipelines {
            // pipeline_indices.push(Vec::new());
            for (i, entity) in entities.iter().enumerate() {
                if let Some(active_material_index) = entity.active_material_id() {
                    if pipeline.id() == &active_material_index {
                        pipeline_indices./* last_mut().unwrap(). */push(i);
                    }
                } else {
                    no_pipeline_indices.push(i);
                }
            }
        }
        no_pipeline_indices.shrink_to_fit();
        (pipeline_indices, no_pipeline_indices)
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
        let mut entities = entities_arc.lock().unwrap();
        let mut non_updated_entities: Vec<usize> = (0..entities.len()).into_iter().collect();
        let camera_bind_group = self.create_camera_bind_group();

        let mats_arc:Arc<Mutex<Vec<MaterialComponent>>> = Arc::new(Mutex::new(Vec::new()));
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

        let (pipeline_entity_indices, no_pipeline_entity_indices) = self.sort_entities_by_index();
        for (pipeline_index, entity_index) in pipeline_entity_indices.iter().enumerate() {
            // render_pass.set_pipeline(&self.pipelines[pipeline_index].pipeline());
            // for entity_index in entity_indices.iter() {
                let mut entity = &mut entities[*entity_index];

                let entity_materials = entity.materials().clone();

                if let Some(active_material_index) = &entity.active_material_index() {
                    let active_mat = &entity_materials[*active_material_index];
                    // let active_mat = &mats[0];
                    render_pass.set_bind_group(0, active_mat.bind_group(), &[]);
                }
            // }
        } 

        /* for pipeline in &self.pipelines {
            // render_pass.set_bind_group(1, &camera_bind_group, &[]);
            for entity in entities.iter_mut() {
                if entity.active_material_id() != Some(pipeline.id().clone()) {
                    continue;
                }
                let entity_materials = entity.materials().clone();

                if let Some(active_material_index) = &entity.active_material_index() {
                    let active_mat = &entity_materials[*active_material_index];
                    // render_pass.set_bind_group(0, active_mat.bind_group(), &[]);
                }
                // let components = entity.components_mut();
                /* for (i, component) in components.iter_mut().enumerate() {
                    component.update(
                        self.device.clone(),
                        self.queue.clone(),
                        entities_arc.clone(),
                        engine_details.clone(),
                        &mut render_pass,
                    );
                    non_updated_entities
                        .remove(non_updated_entities.iter().position(|&el| el == i).unwrap());
                } */
            }
        } */

        /* for index in non_updated_entities.iter() {
            let mut entity = &mut entities[*index];

            let mut components = entity.components_mut();
            for (i, component) in components.iter_mut().enumerate() {
                component.update(
                    self.device.clone(),
                    self.queue.clone(),
                    entities_arc,
                    engine_details,
                    &mut render_pass,
                );
                non_updated_entities
                    .remove(non_updated_entities.iter().position(|&el| el == i).unwrap());
            }
        } */
    }

    pub fn initialize(&mut self) {
        let entities_arc = self.entities.clone();
        let mut entities = entities_arc.lock().unwrap();
        for entity in entities.iter_mut() {
            let components = entity.components_mut();
            for component in components.iter_mut() {
                component.initialize(
                    self.device.clone(),
                    self.queue.clone(),
                    entities_arc.clone(),
                );
            }
        }
    }

    pub fn create_camera_bind_group(&self) -> BindGroup {
        let device = self.device.clone();

        if let Some(active_camera_id) = self.active_camera_id {
            let entities_arc = self.entities.clone();
            let entities = entities_arc.lock().unwrap();
            let camera_entity_index = entities
                .deref()
                .into_iter()
                .position(|entity| entity.get_id() == &active_camera_id)
                .unwrap();
            let camera_entity = &entities[camera_entity_index];

            let camera_entity_components = &camera_entity.components();

            // Do some error checking, make sure these components exist
            let camera_component_index =
                camera_entity.get_indices_of_components::<CameraComponent>()[0];
            let transform_components_indices =
                camera_entity.get_indices_of_components::<TransformComponent>();
            let position = match transform_components_indices.len() {
                0 => na::Vector3::new(0.0, 0.0, 0.0),
                _ => {
                    let transform_component = camera_entity_components
                        [transform_components_indices[0]]
                        .as_any()
                        .downcast_ref::<TransformComponent>()
                        .unwrap();
                    *transform_component.position()
                }
            };
            let cam = camera_entity_components[camera_component_index]
                .as_any()
                .downcast_ref::<CameraComponent>()
                .unwrap();

            return cam.create_camera_bind_group(device.clone(), position);
        }

        let cam = CameraComponent::new(u32::MAX);
        cam.create_camera_bind_group(device.clone(), na::Vector3::new(0.0, 0.0, 0.0))
    }
}
