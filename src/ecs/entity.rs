#![allow(unused)]
use std::collections::HashMap;

use cool_utils::data_structures::tree::Tree;

use super::component::ComponentSystem;

#[derive(Debug)]
pub struct Entity {
    id: Vec<usize>,
}

impl Entity {
    pub fn new(id: Vec<usize>) -> Self {
        Self { id }
    }

    pub fn get_id<'a>(&'a self) -> &'a Vec<usize> {
        &self.id
    }
}

pub struct EntityManager {
    entities: Tree<Entity>,
    component_map: HashMap<Vec<usize>, Vec<Box<dyn ComponentSystem>>>,
}

impl EntityManager {
    pub fn new() -> Self {
        let root_index = vec![0];
        let mut component_map: HashMap<Vec<usize>, Vec<Box<dyn ComponentSystem>>> = HashMap::new();
        component_map.insert(root_index.clone(), Vec::new());
        Self {
            entities: Tree::new(Entity::new(root_index)),
            component_map,
        }
    }
}

impl ComponentSystem for EntityManager {
    fn initialize(&mut self) {
        for (_, components) in self.component_map.iter_mut() {
            for component in components {
                component.initialize();
            }
        }
    }
    fn update(&mut self) {
        for (_, components) in self.component_map.iter_mut() {
            for component in components {
                component.update();
            }
        }
    }
}
