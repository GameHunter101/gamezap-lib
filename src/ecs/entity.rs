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
    components: Vec<Component>,
    materials: Vec<MaterialComponent>,
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
            components,
            materials,
        }
    }

    pub fn get_id(&self) -> &EntityId {
        &self.id
    }

    pub fn active_material_id(&self) -> Option<MaterialId> {
        let mats = &self.materials;
        if mats.len() == 0 {
            return None;
        }
        for mat in mats.iter() {
            if mat.enabled() {
                return Some(mat.id().clone());
            }
        }
        return None;
    }

    pub fn active_material_index(&self) -> Option<usize> {
        let mats = &self.materials;
        if mats.len() == 0 {
            return None;
        }
        for (i, mat) in mats.iter().enumerate() {
            if mat.enabled() {
                return Some(i);
            }
        }
        return None;
    }

    pub fn components(&self) -> &Vec<Component> {
        &self.components
    }

    pub fn components_mut(&mut self) -> &mut Vec<Component> {
        &mut self.components
    }

    pub fn materials(&self) -> &Vec<MaterialComponent> {
        &self.materials
    }

    pub fn get_indices_of_components<T: ComponentSystem + Any>(&self) -> Vec<usize> {
        self.components
            .iter()
            .enumerate()
            .map(|(i, comp)| (i, comp.as_any().downcast_ref::<T>()))
            .filter(|(i, comp)| comp.is_some())
            .map(|(i, _)| i)
            .collect()
    }
}
