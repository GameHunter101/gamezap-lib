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
    sync::{Arc, Mutex, MutexGuard},
};

use cool_utils::data_structures::tree::Tree;
use smaa::SmaaTarget;
use wgpu::{CommandEncoderDescriptor, Device, Queue, RenderPass, Surface, TextureFormat};

use nalgebra as na;

use crate::pipeline::Pipeline;

use super::{
    component::{CameraComponent, Component, ComponentSystem, MaterialComponent, MaterialId},
    entity::EntityId,
};

pub struct Scene {
    entities: Arc<Mutex<Vec<Entity>>>,
    total_entites_created: u32,
    pipelines: Vec<Pipeline>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    color_format: TextureFormat,
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
            if !self.does_pipeline_exist(mat_id) {
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

    fn does_pipeline_exist(&self, id: MaterialId) -> bool {
        for pipeline in self.pipelines {
            if pipeline.id() == &id {
                return true;
            }
        }
        return false;
    }

    pub fn update(
        &mut self,
        engine_details: Arc<Mutex<EngineDetails>>,
        smaa_target: Arc<Mutex<SmaaTarget>>,
        surface: Surface,
        depth_texture: Texture,
    ) {
        let device = self.device.clone();
        let queue = self.queue.clone();

        let entities_arc = self.entities.clone();
        let entities = entities_arc.lock().unwrap();
        let mut non_updated_entities: Vec<usize> = (0..entities.len()).into_iter().collect();

        let output = surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut smaa_binding = smaa_target.lock().unwrap();
        let smaa_frame = smaa_binding.start_frame(&device, &queue, &view);

        let encoder = self
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

        for pipeline in &self.pipelines {
            for entity in entities.iter() {
                let components = entity.components();
                for component in components.lock().unwrap().iter_mut() {
                    component.update(
                        self.device.clone(),
                        self.queue.clone(),
                        entities_arc,
                        engine_details,
                        &mut render_pass,
                    );
                }
            }
        }
    }
}
