use crate::{
    ecs::entity::Entity,
    model::{Mesh, Vertex, VertexData},
    texture::Texture,
    EngineDetails,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cool_utils::data_structures::tree::Tree;
use wgpu::{Device, Queue, Surface};

use crate::pipeline::Pipeline;

use super::{
    component::{EntityComponentGroup, MaterialId},
    entity::EntityId,
};
pub struct Scene {
    entities: Tree<Entity>,
    components: Arc<Mutex<HashMap<EntityId, Arc<Mutex<EntityComponentGroup>>>>>,
    pipelines: Arc<Mutex<Vec<(MaterialId, Pipeline)>>>,
}

impl Scene {
    pub fn new() -> Self {
        let root_index = vec![0];
        Self {
            entities: Tree::new(Entity::new(root_index)),
            components: Arc::new(Mutex::new(HashMap::new())),
            pipelines: Arc::from(Mutex::new(Vec::new())),
        }
    }

    fn initialize_material(
        &mut self,
        entity: &EntityId,
        device: Arc<Device>,
        color_format: wgpu::TextureFormat,
    ) {
        let all_components = self.components.lock().unwrap();
        let components_arc = all_components.get(entity).unwrap();
        let components = components_arc.lock().unwrap();
        let material_id = components.get_active_material();
        if let Some(mat_id) = material_id {
            let is_new_material = self
                .pipelines
                .lock()
                .unwrap()
                .iter()
                .position(|(id, _)| id == mat_id)
                .is_none();

            if is_new_material {
                let new_pipeline_layout = Pipeline::create_pipeline_layout(mat_id, device.clone());
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

    fn create_entity(
        &mut self,
        index: Option<EntityId>,
        components: Arc<Mutex<EntityComponentGroup>>,
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

        self.initialize_material(&entity_id, device, color_format);

        entity_id
    }

    pub fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        color_format: wgpu::TextureFormat,
    ) {
        let all_components = self.components.clone();
        for (_, components_arc) in all_components.lock().unwrap().iter_mut() {
            let mut components = components_arc.lock().unwrap();
            for normal_comp in components.get_normal_components_mut() {
                normal_comp.initialize(device.clone(), queue.clone(), components_arc.clone());
            }
            for material_comp in components.get_material_components_mut() {
                self.initialize_material(material_comp.this_entity(), device.clone(), color_format);
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

        for component_group_arc in self.components.lock().unwrap().values_mut().into_iter() {
            let mut component_group = component_group_arc.lock().unwrap();
            if component_group.get_active_material().is_none() {
                for component in component_group.get_normal_components_mut() {
                    component.update(
                        device.clone(),
                        queue.clone(),
                        component_group_arc.clone(),
                        engine_details.clone(),
                    );
                }
            }
        }
        for pipeline_data in pipelines.iter() {
            let (current_pipeline_id, pipeline) = pipeline_data;
            render_pass.set_pipeline(&pipeline.pipeline);
            for component_group_arc in self.components.lock().unwrap().values_mut().into_iter() {
                let mut component_group = component_group_arc.lock().unwrap();
                if let Some(mat_id) = component_group.get_active_material() {
                    if current_pipeline_id == mat_id {
                        for component in component_group.get_normal_components_mut() {
                            component.update(
                                device.clone(),
                                queue.clone(),
                                component_group_arc.clone(),
                                engine_details.clone()
                            );
                        }
                    }
                }
            }
        }
    }
}
