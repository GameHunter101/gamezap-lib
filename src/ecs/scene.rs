use crate::{
    compute::{ComputePipeline, ComputePipelineType},
    ecs::{concepts::ConceptManager, entity::Entity},
    model::{Vertex, VertexData},
    pipeline::PipelineError,
    texture::Texture,
    ui_manager::UiManager,
    EngineDetails, EngineSystems,
};
use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    rc::Rc,
    sync::{Arc, Mutex},
};

use glyphon::{FontSystem, SwashCache, TextAtlas, TextRenderer};
use wgpu::{BindGroup, CommandEncoderDescriptor, Device, Queue, TextureFormat};

use crate::pipeline::Pipeline;

use super::{
    component::{Component, ComponentSystem},
    components::{camera_component::CameraComponent, transform_component::TransformComponent},
    entity::EntityId,
    material::{Material, MaterialId},
};

pub type AllComponents = HashMap<EntityId, Vec<Component>>;
pub type Materials = HashMap<EntityId, (Vec<Material>, usize)>;

// #[derive(Debug)]
pub struct TextState {
    pub font_system: glyphon::FontSystem,
    pub swash_cache: glyphon::SwashCache,
    pub text_viewport: Option<glyphon::Viewport>,
    pub atlas: Option<glyphon::TextAtlas>,
    pub text_renderer: Option<glyphon::TextRenderer>,
    pub text_items: Vec<TextParams>,
}

#[derive(Debug, Clone)]
pub struct TextParams {
    pub metrics: glyphon::Metrics,
    pub text: String,
    pub color: glyphon::Color,
    pub family: glyphon::Family<'static>,
    pub weight: glyphon::Weight,
    pub fancy_render: bool,
    pub top_left_position: (f32, f32),
    pub text_scale: f32,
    pub bounds: glyphon::TextBounds,
    pub default_color: glyphon::Color,
}

impl Debug for TextState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextState")
            .field("font_system", &self.font_system)
            .field("swash_cache", &self.swash_cache)
            .field("text_viewport", &self.text_viewport)
            .field("atlas", &"Text atlas".to_string())
            .field("text_renderer", &"Text renderer".to_string())
            .field("text_areas", &"Text areas".to_string())
            .finish()
    }
}

#[derive(Debug)]
pub struct Scene {
    entities: Arc<Mutex<Vec<Entity>>>,
    total_entities_created: u32,
    pipelines: HashMap<MaterialId, Pipeline>,
    compute_pipelines: Vec<ComputePipeline>,
    components: AllComponents,
    materials: Materials,
    active_camera_id: Option<EntityId>,
    concept_manager: Rc<Mutex<ConceptManager>>,

    pub text_state: TextState,
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

        for component in components.iter_mut() {
            let old_id = component.get_id();
            component.update_metadata(new_entity_id, 0);
            self.concept_manager
                .lock()
                .unwrap()
                .modify_key(old_id, component.get_id());
        }

        if let Some((materials, active_material_index)) = materials {
            self.materials
                .insert(new_entity_id, (materials, active_material_index));
        }
        self.components.insert(new_entity_id, components);
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
        engine_details: Rc<Mutex<EngineDetails>>,
        engine_systems: Rc<Mutex<EngineSystems>>,
        ui_manager: Rc<Mutex<UiManager>>,
    ) {
        let entities_arc = self.entities.clone();
        let entities = entities_arc.lock().unwrap();
        // let components_arc = self.components.clone();

        let new_components = entities
            .iter()
            .map(|entity| {
                if let Some((materials, active_material_index)) = &self.materials.get(entity.id()) {
                    let active_material = &materials[*active_material_index];
                    let active_material_id = active_material.id().clone();
                    self.pipelines
                        .entry(active_material_id.clone())
                        .or_insert_with(|| {
                            Pipeline::new(
                                device.clone(),
                                color_format,
                                &[Vertex::desc(), TransformComponent::desc()],
                                &active_material_id,
                            )
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
                                &self.components,
                                self.concept_manager.clone(),
                                Some(engine_details.clone()),
                                Some(engine_systems.clone()),
                                ui_manager.clone(),
                                &mut self.text_state.text_items,
                            );
                            comp_clone
                        })
                        .collect::<Vec<Component>>(),
                )
            })
            .collect::<HashMap<EntityId, Vec<Component>>>();

        self.components = new_components;

        let cache = glyphon::Cache::new(&device);
        self.text_state.text_viewport = Some(glyphon::Viewport::new(&device, &cache));
        let mut atlas =
            TextAtlas::new(&device, &queue, &cache, wgpu::TextureFormat::Bgra8UnormSrgb);
        self.text_state.text_renderer = Some(TextRenderer::new(
            &mut atlas,
            &device,
            wgpu::MultisampleState::default(),
            None,
        ));
        self.text_state.atlas = Some(atlas);
    }

    pub fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        engine_details: Rc<Mutex<EngineDetails>>,
        engine_systems: Rc<Mutex<EngineSystems>>,
    ) {
        let entities_arc = self.entities.clone();
        let entities = entities_arc.lock().unwrap();

        let mut entities_clone = entities.clone();

        let mut cloned_components = self
            .components
            .iter()
            .map(|(k, v)| {
                (
                    *k,
                    v.iter()
                        .map(|comp| dyn_clone::clone_box(&**comp))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<HashMap<EntityId, Vec<Component>>>();

        for entity in entities.iter() {
            if entity.enabled {
                let entity_components_len = cloned_components
                    .get(entity.id())
                    .unwrap_or(&Vec::<Component>::new())
                    .len();
                for comp_index in 0..entity_components_len {
                    let mut comp =
                        dyn_clone::clone_box(&*cloned_components[entity.id()][comp_index]);
                    comp.update(
                        device.clone(),
                        queue.clone(),
                        &mut cloned_components,
                        engine_details.clone(),
                        engine_systems.clone(),
                        self.concept_manager.clone(),
                        self.active_camera_id,
                        &mut entities_clone,
                        self.materials.get_mut(entity.id()),
                        &mut self.compute_pipelines,
                        &mut self.text_state.text_items,
                    );
                    let map_ref = cloned_components
                        .get_mut(entity.id())
                        .unwrap()
                        .get_mut(comp_index)
                        .unwrap();
                    *map_ref = comp
                }
            }
        }

        for compute_pipeline in &self.compute_pipelines {
            compute_pipeline.run_compute_shader(&device, &queue);
        }

        self.components = cloned_components;
        self.entities = Arc::new(Mutex::new(entities_clone));
    }

    pub fn ui_draw(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        engine_details: Rc<Mutex<EngineDetails>>,
        engine_systems: Rc<Mutex<EngineSystems>>,
        ui_manager: Rc<Mutex<UiManager>>,
    ) {
        let entities_arc = self.entities.clone();
        let entities = entities_arc.lock().unwrap();

        let mut manager = ui_manager.lock().unwrap();
        let context_arc = manager.imgui_context.clone();
        let mut context = context_arc.lock().unwrap();
        let ui_frame = context.new_frame();
        let mut cloned_components = self
            .components
            .iter()
            .map(|(k, v)| {
                (
                    *k,
                    v.iter()
                        .map(|comp| dyn_clone::clone_box(&**comp))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<HashMap<EntityId, Vec<Component>>>();

        for entity in entities.iter() {
            if entity.enabled {
                let entity_components_len = cloned_components
                    .get(entity.id())
                    .unwrap_or(&Vec::<Component>::new())
                    .len();
                for comp_index in 0..entity_components_len {
                    let mut comp =
                        dyn_clone::clone_box(&*cloned_components[entity.id()][comp_index]);
                    comp.ui_draw(
                        device.clone(),
                        queue.clone(),
                        &mut manager,
                        ui_frame,
                        &mut cloned_components,
                        self.concept_manager.clone(),
                        engine_details.clone(),
                        engine_systems.clone(),
                    );
                    let map_ref = cloned_components
                        .get_mut(entity.id())
                        .unwrap()
                        .get_mut(comp_index)
                        .unwrap();
                    *map_ref = comp
                }
            }
        }

        self.components = cloned_components;
    }

    pub fn render(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        depth_texture: Arc<Texture>,
        window_size: (u32, u32),
        engine_details: &EngineDetails,
        engine_systems: &EngineSystems,
        smaa_frame: smaa::SmaaFrame,
        output: wgpu::SurfaceTexture,
        clear_color: wgpu::Color,
        ui_manager: Rc<Mutex<UiManager>>,
    ) {
        let entities_arc = self.entities.clone();
        let entities = entities_arc.lock().unwrap();

        let camera_bind_group = self.create_camera_bind_group(
            device.clone(),
            queue.clone(),
            window_size,
            ui_manager.clone(),
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Scene Encoder"),
        });

        let mut default_transform = TransformComponent::default(self.concept_manager.clone());
        default_transform.initialize(
            device.clone(),
            queue.clone(),
            &self.components,
            self.concept_manager.clone(),
            None,
            None,
            ui_manager.clone(),
            &mut self.text_state.text_items,
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Scene Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &smaa_frame,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(mask) = &engine_details.render_mask {
                render_pass.set_viewport(mask.x, mask.y, mask.width, mask.height, 0.0, 1.0);
            }

            render_pass.set_bind_group(1, &camera_bind_group, &[]);

            for (pipeline_id, pipeline) in &self.pipelines {
                render_pass.set_pipeline(pipeline.pipeline());

                for entity in entities.iter() {
                    if entity.enabled {
                        let entity_materials = self.materials.get(entity.id());
                        if let Some((materials, active_material_index)) = entity_materials {
                            let active_material = &materials[*active_material_index];
                            if active_material.id() == pipeline_id {
                                render_pass.set_bind_group(
                                    0,
                                    active_material.texture_bind_group(),
                                    &[],
                                );
                                if let Some(uniform_buffer_bind_group) =
                                    active_material.uniform_buffer_bind_group()
                                {
                                    render_pass.set_bind_group(
                                        2,
                                        &uniform_buffer_bind_group.0,
                                        &[],
                                    );
                                }

                                default_transform.render(
                                    device.clone(),
                                    queue.clone(),
                                    &mut render_pass,
                                    &self.components,
                                    self.concept_manager.clone(),
                                    engine_details,
                                    engine_systems,
                                );

                                // render_pass.set_vertex_buffer(1, default_transform_buffer.slice(..));
                                let components_opt = self.components.get(entity.id());
                                if let Some(components) = components_opt {
                                    let ordered_components =
                                        Self::get_component_render_order(components);
                                    for component in ordered_components.iter() {
                                        component.render(
                                            device.clone(),
                                            queue.clone(),
                                            &mut render_pass,
                                            &self.components,
                                            self.concept_manager.clone(),
                                            engine_details,
                                            engine_systems,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        smaa_frame.resolve();

        if let Some(text_renderer) = &mut self.text_state.text_renderer {
            self.text_state.text_viewport.as_mut().unwrap().update(
                &queue,
                glyphon::Resolution {
                    width: window_size.0,
                    height: window_size.1,
                },
            );

            let buffers = self
                .text_state
                .text_items
                .iter()
                .map(|params| {
                    let mut buffer =
                        glyphon::Buffer::new(&mut self.text_state.font_system, params.metrics);

                    buffer.set_size(
                        &mut self.text_state.font_system,
                        Some(window_size.0 as f32),
                        Some(window_size.1 as f32),
                    );

                    buffer.set_text(
                        &mut self.text_state.font_system,
                        &params.text,
                        glyphon::Attrs::new()
                            .color(params.color)
                            .family(params.family)
                            .weight(params.weight),
                        if params.fancy_render {
                            glyphon::Shaping::Advanced
                        } else {
                            glyphon::Shaping::Basic
                        },
                    );

                    buffer.shape_until_scroll(&mut self.text_state.font_system, false);
                    buffer
                })
                .collect::<Vec<_>>();

            let text_areas = self
                .text_state
                .text_items
                .iter()
                .enumerate()
                .map(|(i, params)| glyphon::TextArea {
                    buffer: &buffers[i],
                    left: params.top_left_position.0,
                    top: params.top_left_position.1,
                    scale: params.text_scale,
                    bounds: params.bounds,
                    default_color: params.default_color,
                    custom_glyphs: &[],
                });

            text_renderer
                .prepare(
                    &device,
                    &queue,
                    &mut self.text_state.font_system,
                    self.text_state.atlas.as_mut().unwrap(),
                    self.text_state.text_viewport.as_ref().unwrap(),
                    text_areas,
                    &mut self.text_state.swash_cache,
                )
                .unwrap();
        }

        let ui_manager = ui_manager.lock().unwrap();
        let mut renderer = ui_manager.imgui_renderer.lock().unwrap();
        let mut context = ui_manager.imgui_context.lock().unwrap();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut ui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(text_renderer) = self.text_state.text_renderer.as_ref() {
                text_renderer
                    .render(
                        self.text_state.atlas.as_ref().unwrap(),
                        self.text_state.text_viewport.as_ref().unwrap(),
                        &mut ui_render_pass,
                    )
                    .unwrap();
            }

            self.render_ui(
                device,
                queue.clone(),
                &mut renderer,
                &mut context,
                &mut ui_render_pass,
            );
        }

        drop(renderer);
        drop(context);

        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.text_state.atlas.as_mut().unwrap().trim();
    }

    fn get_component_render_order(components: &[Component]) -> Vec<&Component> {
        let mut render_orders = components
            .iter()
            .enumerate()
            .map(|(i, comp)| (i, comp.render_order()))
            .collect::<Vec<_>>();
        render_orders.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        render_orders
            .iter()
            .map(|(i, _)| &components[*i])
            .collect::<Vec<_>>()
    }

    fn render_ui<'c: 'd, 'd>(
        &self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        renderer: &'c mut imgui_wgpu::Renderer,
        context: &'c mut imgui::Context,
        rpass: &'d mut wgpu::RenderPass<'c>,
    ) {
        renderer
            .render(context.render(), &queue, &device, rpass)
            .unwrap();
    }

    pub fn create_camera_bind_group(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        window_size: (u32, u32),
        ui_manager: Rc<Mutex<UiManager>>,
    ) -> BindGroup {
        if let Some(active_camera_id) = self.active_camera_id {
            let camera_component =
                Scene::get_component::<CameraComponent>(&self.components[&active_camera_id]);
            let bind_group = camera_component.unwrap().create_camera_bind_group(device);
            return bind_group;
        }

        let mut cam = CameraComponent::new_2d(self.concept_manager.clone(), window_size);
        cam.initialize(
            device.clone(),
            queue,
            &self.components,
            self.concept_manager.clone(),
            None,
            None,
            ui_manager,
            &mut self.text_state.text_items,
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

    pub fn get_components(&self) -> &AllComponents {
        &self.components
    }

    pub fn get_components_mut(&mut self) -> &mut AllComponents {
        &mut self.components
    }

    pub fn set_active_camera(&mut self, entity_id: EntityId) {
        self.active_camera_id = Some(entity_id);
    }

    pub fn get_active_camera(&self) -> Option<EntityId> {
        self.active_camera_id
    }

    pub fn get_concept_manager(&self) -> Rc<Mutex<ConceptManager>> {
        self.concept_manager.clone()
    }

    pub fn create_compute_pipeline(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        shader_path: &str,
        workgroup_size: (u32, u32, u32),
        pipeline_type: ComputePipelineType,
    ) -> Result<usize, PipelineError> {
        let this_compute_index = self.compute_pipelines.len();
        let shader_module_descriptor = Pipeline::load_shader_module_descriptor(shader_path)?;
        let pipeline = ComputePipeline::new(
            device,
            queue,
            shader_module_descriptor,
            pipeline_type,
            this_compute_index,
            workgroup_size,
        );
        self.compute_pipelines.push(pipeline);
        Ok(this_compute_index)
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            entities: Arc::new(Mutex::new(Vec::new())),
            total_entities_created: 0,
            pipelines: HashMap::new(),
            compute_pipelines: Vec::new(),
            components: HashMap::new(),
            materials: HashMap::new(),
            active_camera_id: None,
            concept_manager: Rc::new(Mutex::new(ConceptManager::default())),

            text_state: TextState {
                font_system: FontSystem::new(),
                swash_cache: SwashCache::new(),
                text_viewport: None,
                atlas: None,
                text_renderer: None,
                text_items: Vec::new(),
            },
        }
    }
}
