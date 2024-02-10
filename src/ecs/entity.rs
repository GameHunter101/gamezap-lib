#![allow(unused)]
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cool_utils::data_structures::tree::Tree;
use wgpu::{Device, Queue, Surface};

use crate::pipeline::Pipeline;

use super::component::{AsAny, Component, ComponentSystem, Material, MaterialId};

pub type EntityId = u32;

#[derive(Debug)]
pub struct Entity {
    id: EntityId,
    enabled: bool,
    parent: EntityId,
    children: Vec<EntityId>,
}

impl Entity {
    pub fn new(
        id: EntityId,
        enabled: bool,
        parent: EntityId,
        children: Vec<EntityId>,
    ) -> Self {
        Self {
            id,
            enabled,
            parent,
            children,
        }
    }

    pub fn id(&self) -> &EntityId {
        &self.id
    }
}
