use crate::{
    ecs::{concepts::ConceptManager, entity::Entity},
    model::{Vertex, VertexData},
    texture::Texture,
    EngineDetails, EngineSystems,
};
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use wgpu::{BindGroup, CommandEncoderDescriptor, Device, Queue, TextureFormat};

use crate::pipeline::Pipeline;

use super::{
    component::{Component, ComponentSystem},
    components::{camera_component::CameraComponent, transform_component::TransformComponent},
    entity::EntityId,
    material::{Material, MaterialId},
};

pub type AllComponents = Arc<HashMap<EntityId, Vec<Component>>>;
pub type Materials = Arc<Mutex<HashMap<EntityId, (Vec<Material>, usize)>>>;

pub struct Scene {
    entities: Arc<Mutex<Vec<Entity>>>,
    total_entities_created: u32,
    pipelines: HashMap<MaterialId, Pipeline>,
    components: AllComponents,
    materials: Materials,
    active_camera_id: Option<EntityId>,
    concept_manager: Arc<Mutex<ConceptManager>>,
}

#[allow(clippy::too_many_arguments)]
impl Scene {
    pub fn create_entity(
        &mut self,
        parent: EntityId,
        enabled: bool,
        mut components: Vec<Component>,
        materials: Option<(Vec<Material>, usize)>,
    ) -> EntityId {
        let new_entity_id = self.total_entities_created;
        let new_entity = Entity::new(new_entity_id, enabled, parent, Vec::new());

        let mut concept_manager = self.concept_manager.lock().unwrap();

        for component in components.iter_mut() {
            let old_id = component.get_id();
            component.update_metadata(new_entity_id, 0);
            concept_manager.modify_key(old_id, component.get_id());
        }

        if let Some((materials, active_material_index)) = materials {
            self.materials
                .lock()
                .unwrap()
                .insert(new_entity_id, (materials, active_material_index));
        }
        Arc::get_mut(&mut self.components)
            .unwrap()
            .insert(new_entity_id, components);
        let entities = self.entities.clone();
        entities.lock().unwrap().push(new_entity);
        self.total_entities_created += 1;
        new_entity_id
    }

    pub fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        color_format: TextureFormat,
    ) {
        let entities_arc = self.entities.clone();
        let entities = entities_arc.lock().unwrap();
        // let components_arc = self.components.clone();
        let materials_arc = self.materials.clone();
        let materials = materials_arc.lock().unwrap();

        let new_components = entities
            .iter()
            .map(|entity| {
                if let Some((materials, active_material_index)) = materials.get(entity.id()) {
                    let active_material = &materials[*active_material_index];
                    let active_material_id = active_material.id().clone();
                    self.pipelines
                        .entry(active_material_id.clone())
                        .or_insert_with(|| {
                            let new_pipeline = Pipeline::new(
                                device.clone(),
                                color_format,
                                &[Vertex::desc(), TransformComponent::desc()],
                                &active_material_id,
                            );
                            new_pipeline
                        });
                }
                (
                    *entity.id(),
                    self.components
                        .get(entity.id())
                        .unwrap_or(&Vec::<Component>::new())
                        .iter()
                        .map(|comp| {
                            let mut comp_clone = dyn_clone::clone_box(&**comp);
                            comp_clone.initialize(
                                device.clone(),
                                queue.clone(),
                                self.components.clone(),
                                self.concept_manager.clone(),
                            );
                            comp_clone
                        })
                        .collect::<Vec<Component>>(),
                )
            })
            .collect::<HashMap<EntityId, Vec<Component>>>();

        *Arc::get_mut(&mut self.components).unwrap() = new_components;
    }

    pub fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        engine_details: Arc<Mutex<EngineDetails>>,
        engine_systems: Arc<Mutex<EngineSystems>>,
    ) {
        let entities_arc = self.entities.clone();
        let entities = entities_arc.lock().unwrap();

        let new_components = entities
            .iter()
            .map(|entity| {
                let components_futures = self
                    .components
                    .get(entity.id())
                    .unwrap_or(&Vec::<Component>::new())
                    .iter()
                    .map(|comp| {
                        tokio::spawn(async move {
                            let mut comp_clone = dyn_clone::clone_box(&**comp);
                            comp_clone.update(
                                device.clone(),
                                queue.clone(),
                                self.components.clone(),
                                engine_details.clone(),
                                engine_systems.clone(),
                                self.concept_manager.clone(),
                                self.active_camera_id,
                            );
                            comp_clone
                        })
                    })
                    .collect::<Vec<_>>();
                for future in components_futures {
                    future.await.unwrap();
                }
                (*entity.id(), components_futures)
            })
            .collect::<HashMap<EntityId, Vec<Component>>>();

        *Arc::get_mut(&mut self.components).unwrap() = new_components;
    }

    pub fn render(
        &self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        depth_texture: Arc<Texture>,
        window_size: (u32, u32),
        engine_details: Arc<Mutex<EngineDetails>>,
        engine_systems: Arc<Mutex<EngineSystems>>,
        smaa_frame: smaa::SmaaFrame,
        output: wgpu::SurfaceTexture,
    ) {
        let entities_arc = self.entities.clone();
        let entities = entities_arc.lock().unwrap();

        let materials = self.materials.lock().unwrap();

        let camera_bind_group =
            self.create_camera_bind_group(device.clone(), queue.clone(), window_size);
        let components_arc = self.components.clone();

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Scene Encoder"),
        });

        let mut default_transform = TransformComponent::default(self.concept_manager.clone());
        default_transform.initialize(
            device.clone(),
            queue.clone(),
            self.components.clone(),
            self.concept_manager.clone(),
        );

        let concept_manager = &*self.concept_manager.lock().unwrap();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Scene Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &smaa_frame,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.7,
                            g: 0.2,
                            b: 0.2,
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

            if let Some(mask) = &engine_details.lock().unwrap().render_mask {
                render_pass.set_viewport(mask.x, mask.y, mask.width, mask.height, 0.0, 1.0);
            }

            render_pass.set_bind_group(1, &camera_bind_group, &[]);

            for (pipeline_id, pipeline) in &self.pipelines {
                render_pass.set_pipeline(pipeline.pipeline());

                for entity in entities.iter() {
                    let entity_materials = materials.get(entity.id());
                    if let Some((materials, active_material_index)) = entity_materials {
                        let active_material = &materials[*active_material_index];
                        if active_material.id() == pipeline_id {
                            render_pass.set_bind_group(0, active_material.bind_group(), &[]);

                            default_transform.render(
                                device.clone(),
                                queue.clone(),
                                &mut render_pass,
                                &components_arc,
                                concept_manager,
                                engine_details.clone(),
                                engine_systems.clone(),
                            );

                            // render_pass.set_vertex_buffer(1, default_transform_buffer.slice(..));
                            for component in components_arc.get(entity.id()).unwrap().iter() {
                                component.render(
                                    device.clone(),
                                    queue.clone(),
                                    &mut render_pass,
                                    &components_arc,
                                    concept_manager,
                                    engine_details.clone(),
                                    engine_systems.clone(),
                                );
                            }
                        }
                    }
                }
            }
        }
        smaa_frame.resolve();

        let systems = engine_systems.lock().unwrap();
        let mut ui_manager = systems.ui_manager.lock().unwrap();
        let mut renderer = ui_manager.imgui_renderer.lock().unwrap();
        let mut context = ui_manager.imgui_context.lock().unwrap();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut ui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            if ui_manager
                .render_flag
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                self.render_ui(
                    device,
                    queue.clone(),
                    &mut renderer,
                    &mut context,
                    &mut ui_render_pass,
                );
            }
        }

        drop(renderer);
        drop(context);

        ui_manager.clear_render_flag();

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn render_ui<'a: 'b, 'b>(
        &self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        renderer: &'a mut imgui_wgpu::Renderer,
        context: &'a mut imgui::Context,
        rpass: &mut wgpu::RenderPass<'b>,
    ) {
        renderer
            .render(context.render(), &queue, &device, rpass)
            .unwrap();
    }

    pub fn create_camera_bind_group(
        &self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        window_size: (u32, u32),
    ) -> BindGroup {
        let components_arc = self.components.clone();

        if let Some(active_camera_id) = self.active_camera_id {
            let camera_component =
                Scene::get_component::<CameraComponent>(&components_arc[&active_camera_id]);
            let bind_group = camera_component.unwrap().create_camera_bind_group(device);
            return bind_group;
        }

        let mut cam = CameraComponent::new_2d(self.concept_manager.clone(), window_size);
        cam.initialize(
            device.clone(),
            queue,
            self.components.clone(),
            self.concept_manager.clone(),
        );
        cam.create_camera_bind_group(device)
    }

    pub fn get_component<T: ComponentSystem + Any>(components: &[Component]) -> Option<&T> {
        for component in components {
            if let Some(comp) = component.as_any().downcast_ref::<T>() {
                return Some(comp);
            }
        }
        None
    }

    pub fn get_component_mut<T: ComponentSystem + Any>(
        components: &mut [Component],
    ) -> Option<&mut T> {
        for component in components.iter_mut() {
            if let Some(comp) = component.as_any_mut().downcast_mut::<T>() {
                return Some(comp);
            }
        }
        None
    }

    pub fn get_components(&self) -> AllComponents {
        self.components.clone()
    }

    pub fn set_active_camera(&mut self, entity_id: EntityId) {
        self.active_camera_id = Some(entity_id);
    }

    pub fn get_active_camera(&self) -> Option<EntityId> {
        self.active_camera_id
    }

    pub fn get_concept_manager(&self) -> Arc<Mutex<ConceptManager>> {
        self.concept_manager.clone()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            entities: Arc::new(Mutex::new(Vec::new())),
            total_entities_created: 0,
            pipelines: HashMap::new(),
            components: Arc::new(HashMap::new()),
            materials: Arc::new(Mutex::new(HashMap::new())),
            active_camera_id: None,
            concept_manager: Arc::new(Mutex::new(ConceptManager::default())),
        }
    }
}
