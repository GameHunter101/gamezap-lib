#![allow(unused)]
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cool_utils::data_structures::tree::Tree;
use wgpu::{Device, Queue, Surface};

use crate::pipeline::Pipeline;

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

