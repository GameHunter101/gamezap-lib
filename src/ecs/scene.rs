use std::{collections::HashMap, fmt::Debug};

use glyphon::{FontSystem, SwashCache, TextAtlas, TextRenderer, Viewport};
use wgpu::{Device, Queue, RenderPipeline, TextureFormat};
use winit_input_helper::WinitInputHelper;

use super::{
    actions::Action,
    builtin_actions::workload_action::Workload,
    component::{Component, ComponentId},
    entity::{Entity, EntityId},
    material::{Material, MaterialAttachment, MaterialId},
    pipeline::{create_render_pipeline, PipelineDetails, PipelineId},
};

pub struct Scene<'a> {
    components: Vec<Component>,
    entities: HashMap<EntityId, Entity>,
    ui_components: Vec<ComponentId>,
    pipelines: HashMap<PipelineId, RenderPipeline>,
    materials: Vec<Material>,
    pipeline_material_sets: HashMap<PipelineId, Vec<MaterialId>>,
    active_camera_id: Option<ComponentId>,
    total_entities_created: u32,
    font_state: FontState,
    action_queue: Vec<Box<dyn Action<'a> + Send>>,
    workloads: HashMap<ComponentId, &'a mut Workload>,
}

pub struct FontState {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub viewport: glyphon::Viewport,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub text_buffers: Vec<glyphon::Buffer>,
}

impl<'a> Debug for Scene<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("components", &self.components.len())
            .finish()
    }
}

impl<'a> Scene<'a> {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = glyphon::Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        let font_state = FontState {
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            text_buffers: Vec::new(),
        };

        Scene {
            components: Vec::new(),
            entities: HashMap::new(),
            ui_components: Vec::new(),
            pipelines: HashMap::new(),
            materials: Vec::new(),
            pipeline_material_sets: HashMap::new(),
            active_camera_id: None,
            total_entities_created: 0,
            font_state,
            action_queue: Vec::new(),
            workloads: HashMap::new(),
        }
    }

    pub fn initialize(&self) {
        for component in &self.components {
            component.initialize();
        }
    }

    pub async fn update(
        &mut self,
        device: &Device,
        queue: &Queue,
        input_manager: &WinitInputHelper,
    ) {
        let all_components = &mut self.components;
        for i in 0..all_components.len() {
            async_scoped::TokioScope::scope_and_block(|scope| {
                let (components_before, components_after_and_this) = all_components.split_at_mut(i);
                let proc = async move {
                    if let Some((component, components_after)) =
                        components_after_and_this.split_first_mut()
                    {
                        let chain: Vec<&mut Component> = components_before
                            .iter_mut()
                            .chain(components_after.iter_mut())
                            .collect();
                        component.update(device, queue, input_manager, &chain).await;
                    }
                };
                scope.spawn(proc)
            });
        }
    }

    pub fn attach_workload(&mut self, component: ComponentId, workload: &'a mut Workload) {
        self.workloads.insert(component, workload);
    }

    #[tokio::main]
    pub async fn run_workloads(&mut self) {
        let workloads = &mut self.workloads;
        async_scoped::TokioScope::scope_and_block(|scope| {
            for (component, workload) in workloads {
                let proc = async move { (*component, workload.await) };
                scope.spawn(proc);
            }
        });
    }

    pub fn create_material(
        &mut self,
        device: &Device,
        render_format: TextureFormat,
        vertex_shader_path: &'static str,
        fragment_shader_path: &'static str,
        attachments: Vec<MaterialAttachment>,
        pipeline_details: PipelineDetails,
    ) -> &Material {
        let new_material = Material::new(
            device,
            self.materials.len(),
            vertex_shader_path,
            fragment_shader_path,
            attachments,
        );
        if let std::collections::hash_map::Entry::Vacant(e) = self
            .pipelines
            .entry((vertex_shader_path, fragment_shader_path))
        {
            e.insert(create_render_pipeline(
                device,
                vertex_shader_path,
                fragment_shader_path,
                new_material.bind_group_layouts(),
                render_format,
                pipeline_details,
            ));
        }

        let id = new_material.id();

        self.materials.push(new_material);

        if let Some(indices) = self
            .pipeline_material_sets
            .get_mut(&(vertex_shader_path, fragment_shader_path))
        {
            indices.push(id);
        }

        self.materials
            .last()
            .expect("Failed to retrieve the newly created material")
    }

    pub fn pipelines(&self) -> &HashMap<PipelineId, RenderPipeline> {
        &self.pipelines
    }

    pub fn get_pipeline_materials(&self, pipeline_id: PipelineId) -> Vec<&Material> {
        let material_ids = self.pipeline_material_sets.get(&pipeline_id);
        match material_ids {
            Some(material_ids) => self
                .materials
                .iter()
                .filter(|mat| material_ids.contains(&mat.id()))
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn get_components_per_material(&self) -> HashMap<MaterialId, Vec<&Component>> {
        let mut output: HashMap<MaterialId, Vec<&Component>> = self
            .materials
            .iter()
            .map(|mat| (mat.id(), Vec::new()))
            .collect();

        for component in &self.components {
            let component_parent_entity_id = component.parent_entity();
            let parent_entity_material_id =
                self.entities[&component_parent_entity_id].active_material();
            output
                .get_mut(&parent_entity_material_id)
                .unwrap()
                .push(component);
        }

        output
    }

    pub fn create_entity(
        &mut self,
        parent: Option<EntityId>,
        components: Vec<Component>,
        material: MaterialId,
        is_enabled: bool,
    ) -> EntityId {
        let entity = Entity::new(
            self.total_entities_created + 1,
            Vec::new(),
            parent.unwrap_or(0),
            is_enabled,
            material,
        );
        let id = entity.id();

        self.entities.insert(id, entity);
        self.total_entities_created += 1;

        for component in components {
            let insert_index = self
                .components
                .iter()
                .position(|comp| comp.rendering_order() >= component.rendering_order())
                .unwrap_or(self.components.len());
            self.components.insert(insert_index, component);
            self.components.last_mut().unwrap().set_parent_entity(id);
        }

        id
    }
}
