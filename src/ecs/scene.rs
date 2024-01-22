use crate::ecs::entity::Entity;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cool_utils::data_structures::tree::Tree;
use wgpu::{Device, Queue, Surface};

use crate::pipeline::Pipeline;

use super::component::{Component, ComponentSystem};
pub struct Scene {
    index: Vec<usize>,
    entities: Tree<Entity>,
    component_map: Arc<Mutex<HashMap<Vec<usize>, Vec<Component>>>>,
    render_pipelines: Arc<Mutex<HashMap<(String, String, usize), Pipeline>>>,
}

impl Scene {
    pub fn new() -> Self {
        let root_index = vec![0];
        let mut component_map: HashMap<Vec<usize>, Vec<Component>> = HashMap::new();
        component_map.insert(root_index.clone(), Vec::new());
        Self {
            index: vec![0],
            entities: Tree::new(Entity::new(root_index)),
            component_map: Arc::new(Mutex::new(component_map)),
            render_pipelines: Arc::from(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_entity(
        &mut self,
        index: Option<Vec<usize>>,
        components: Vec<Component>,
    ) -> Vec<usize> {
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
            .append_at_depth(index, Entity::new(entity_id.clone()));
        self.component_map
            .clone()
            .lock()
            .unwrap()
            .insert(entity_id.clone(), components);
        entity_id
    }
}

impl ComponentSystem for Scene {
    fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        _: &mut Vec<Component>,
    ) {
        for (_, components) in self.component_map.clone().lock().unwrap().iter_mut() {
            for component in components {
                let map_arc = self.component_map.clone();
                let mut component_map = map_arc.lock().unwrap();
                let sibling_components = component_map.get_mut(component.this_entity()).unwrap();
                match component {
                    Component::Normal(comp) => {
                        comp.initialize(device.clone(), queue.clone(), sibling_components)
                    }
                    _ => {}
                };
            }
        }
    }
    fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        _: &mut Vec<Component>,
        smaa_target: Arc<Mutex<smaa::SmaaTarget>>,
        surface: Arc<Surface>,
        depth_texture: Arc<crate::texture::Texture>,
    ) {
        for (_, components) in self.component_map.clone().lock().unwrap().iter_mut() {
            for component in components {
                let map_arc = self.component_map.clone();
                let mut component_map = map_arc.lock().unwrap();
                let sibling_components = component_map.get_mut(component.this_entity()).unwrap();
                match component {
                    Component::Normal(comp) => comp.update(
                        device.clone(),
                        queue.clone(),
                        sibling_components,
                        smaa_target.clone(),
                        surface.clone(),
                        depth_texture.clone(),
                    ),
                    _ => {}
                };
            }
        }
    }
    fn this_entity(&self) -> &Vec<usize> {
        &self.index
    }
}
