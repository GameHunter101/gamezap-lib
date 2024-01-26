#![allow(unused)]
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cool_utils::data_structures::tree::Tree;
use wgpu::{Device, Queue, Surface};

use crate::pipeline::Pipeline;

use super::component::ComponentSystem;

pub type EntityId = Vec<usize>;

#[derive(Debug)]
pub struct Entity {
    id: EntityId
}

impl Entity {
    pub fn new(id: EntityId) -> Self {
        Self { id }
    }

    pub fn get_id(&self) -> &EntityId {
        &self.id
    }
}

