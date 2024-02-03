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
use wgpu::{Device, Queue, RenderPass, Surface};

use nalgebra as na;

use crate::pipeline::Pipeline;

use super::{
    component::{CameraComponent, ComponentSystem, MaterialComponent, MaterialId},
    entity::EntityId,
};

pub struct Scene {
    entities: Tree<Entity>,
    components: Arc<Mutex<HashMap<EntityId, Vec<Box<dyn ComponentSystem>>>>>,
    pipelines: Arc<Mutex<Vec<(MaterialId, Pipeline)>>>,
    materials: Arc<Mutex<HashMap<EntityId, (Vec<MaterialComponent>, usize)>>>,
    active_camera_id: Option<EntityId>,
    cameras: Arc<Mutex<HashMap<EntityId, CameraComponent>>>,
}

impl Scene {
    pub fn new() -> Self {
        let root_index = vec![0];
        Self {
            entities: Tree::new(Entity::new(root_index)),
            components: Arc::new(Mutex::new(HashMap::new())),
            pipelines: Arc::from(Mutex::new(Vec::new())),
            materials: Arc::new(Mutex::new(HashMap::new())),
            active_camera_id: None,
            cameras: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn initialize_material(
        &mut self,
        entity_id: &EntityId,
        device: Arc<Device>,
        color_format: wgpu::TextureFormat,
    ) {
        let all_materials = self.materials.lock().unwrap();
        let materials = all_materials.get(entity_id);
        if let Some(materials) = materials {
            let active_material = materials.0.get(materials.1);
            if let Some(active_material) = active_material {
                let mat_id = active_material.id();

                let is_new_material = self
                    .pipelines
                    .lock()
                    .unwrap()
                    .iter()
                    .position(|(id, _)| id == mat_id)
                    .is_none();

                if is_new_material {
                    let new_pipeline_layout =
                        Pipeline::create_pipeline_layout(mat_id, device.clone());
                    let new_pipeline = Pipeline::new(
                        &format!("{:?} Pipeline", mat_id),
                        device,
                        &new_pipeline_layout,
                        color_format,
                        Some(Texture::DEPTH_FORMAT),
                        &[Vertex::desc(), Mesh::desc()],
                        Pipeline::load_shader_module_descriptor(&mat_id.0),
                        Pipeline::load_shader_module_descriptor(&mat_id.1),
                    );
                }
            }
        }
    }

    fn create_entity(
        &mut self,
        index: Option<EntityId>,
        components: Vec<Box<dyn ComponentSystem>>,
        materials: Vec<MaterialComponent>,
        active_material: Option<usize>,
        device: Arc<Device>,
        color_format: wgpu::TextureFormat,
    ) -> EntityId {
        let index = match index {
            Some(i) => i,
            None => vec![0],
        };

        let mut entity_id = index.clone();
        entity_id.push(
            self.entities
                .index_depth(index.clone())
                .unwrap()
                .children()
                .len(),
        );

        self.entities
            .append_at_depth(index, Entity::new(entity_id.clone()))
            .unwrap();

        self.components
            .lock()
            .unwrap()
            .insert(entity_id.clone(), components);

        if let Some(active_mat) = active_material {
            self.materials
                .lock()
                .unwrap()
                .insert(entity_id.clone(), (materials, active_mat));
            self.initialize_material(&entity_id, device, color_format);
        }

        entity_id
    }

    pub fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        color_format: wgpu::TextureFormat,
    ) {
        let all_components = self.components.clone();
        let materials_arc = self.materials.clone();
        let all_materials = materials_arc.lock().unwrap();
        for (entity_id, components) in all_components.lock().unwrap().iter_mut() {
            for normal_comp in components.iter_mut() {
                normal_comp.initialize(
                    device.clone(),
                    queue.clone(),
                    all_components.clone(),
                    &mut self.active_camera_id,
                );
            }
            let materials = all_materials.get(entity_id);
            if let Some((materials, _)) = materials {
                for material in materials {
                    self.initialize_material(material.this_entity(), device.clone(), color_format);
                }
            }
        }
    }

    pub fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        smaa_target: Arc<Mutex<smaa::SmaaTarget>>,
        surface: Arc<Surface>,
        depth_texture: Arc<crate::texture::Texture>,
        engine_details: Arc<Mutex<EngineDetails>>,
    ) {
        let pipelines_clone = self.pipelines.clone();
        let pipelines = pipelines_clone.lock().unwrap();
        let components_arc = self.components.clone();
        let mut all_components = components_arc.lock().unwrap();
        let materials_arc = self.materials.clone();
        let mut all_materials = materials_arc.lock().unwrap();
        let cameras_arc = self.cameras.clone();
        let mut all_cameras = cameras_arc.lock().unwrap();

        let output = surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut smaa_binding = smaa_target.lock().unwrap();
        let smaa_frame = Arc::new(smaa_binding.start_frame(&device, &queue, &view));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("Entity {:?} Command Encoder", [0])),
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("Entity {:?} Render Pass", [0])),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &smaa_frame,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
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

        self.update_plain_entities(
            components_arc.clone(),
            materials_arc.clone(),
            device.clone(),
            queue.clone(),
            engine_details.clone(),
            &mut render_pass,
        );

        for pipeline_data in pipelines.iter() {
            let (current_pipeline_id, pipeline) = pipeline_data;
            render_pass.set_pipeline(&pipeline.pipeline);

            for (entity_id, (component_materials, active_material_index)) in all_materials.iter() {
                let active_material = &component_materials[*active_material_index];
                let active_material_id = active_material.id();
                if active_material_id == current_pipeline_id {
                    render_pass.set_bind_group(0, &active_material.bind_group(), &[]);

                    if let Some(components) = all_components.get_mut(entity_id) {
                        for component in components.iter_mut() {
                            component.update(
                                device.clone(),
                                queue.clone(),
                                components_arc.clone(),
                                engine_details.clone(),
                                &mut render_pass,
                                &mut self.active_camera_id,
                            );
                        }
                    }
                }
            }
        }
    }

    fn update_plain_entities(
        &mut self,
        components_arc: Arc<Mutex<HashMap<Vec<usize>, Vec<Box<dyn ComponentSystem>>>>>,
        materials_arc: Arc<Mutex<HashMap<Vec<usize>, (Vec<MaterialComponent>, usize)>>>,
        device: Arc<Device>,
        queue: Arc<Queue>,
        engine_details: Arc<Mutex<EngineDetails>>,
        render_pass: &mut RenderPass,
    ) {
        let mut all_components = components_arc.lock().unwrap();
        let all_materials = materials_arc.lock().unwrap();
        for (component_id, components) in all_components.iter_mut() {
            if all_materials.get(component_id).is_none() {
                for component in components {
                    component.update(
                        device.clone(),
                        queue.clone(),
                        components_arc.clone(),
                        engine_details.clone(),
                        render_pass,
                        &mut self.active_camera_id,
                    );
                }
            }
        }
    }
}
