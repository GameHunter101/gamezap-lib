#![allow(unused)]
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cool_utils::data_structures::tree::Tree;
use wgpu::{Device, Queue, Surface};

use crate::pipeline::Pipeline;

use super::component::{AsAny, Component, ComponentSystem, MaterialComponent, MaterialId};

pub type EntityId = u32;

#[derive(Debug)]
pub struct Entity {
    id: EntityId,
    enabled: bool,
    parent: EntityId,
    children: Vec<EntityId>,
    components: Arc<Mutex<Vec<Component>>>,
    materials: Arc<Mutex<Vec<MaterialComponent>>>,
}

impl Entity {
    pub fn new(
        id: EntityId,
        enabled: bool,
        parent: EntityId,
        children: Vec<EntityId>,
        components: Vec<Component>,
        materials: Vec<MaterialComponent>,
    ) -> Self {
        Self {
            id,
            enabled,
            parent,
            children,
            components: Arc::new(Mutex::new(components)),
            materials: Arc::new(Mutex::new(materials)),
        }
    }

    pub fn get_id(&self) -> &EntityId {
        &self.id
    }

    pub fn active_material_id(&self) -> Option<MaterialId>{
        let mats = self.materials.lock().unwrap();
        if mats.len() == 0 {
            return None;
        }
        for mat in mats.iter() {
            if mat.enabled() {
                return Some(*mat.id());
            }
        }
        return None;
    }

    pub fn components(&self) -> Arc<Mutex<Vec<Component>>>{
        self.components.clone()
    }
}
