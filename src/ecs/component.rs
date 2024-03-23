use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use wgpu::{Device, Queue, RenderPass};

use crate::EngineDetails;

use super::{concepts::ConceptManager, entity::EntityId, scene::AllComponents};

pub type ComponentId = (EntityId, TypeId, u32);

pub type Component = Box<dyn ComponentSystem>;

#[allow(unused)]
pub trait ComponentSystem: Debug + dyn_clone::DynClone{
    fn register_component(
        &mut self,
        concept_manager: Arc<Mutex<ConceptManager>>,
        data: HashMap<String, Box<dyn Any>>,
    ) {
    }

    fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        component_map: AllComponents,
        concept_manager: Arc<Mutex<ConceptManager>>,
    ) {
    }

    fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        component_map: AllComponents,
        engine_details: Arc<Mutex<EngineDetails>>,
        concept_manager: Arc<Mutex<ConceptManager>>,
    ) {
    }

    fn render<'a: 'b, 'b>(
        &'a self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        render_pass: &mut RenderPass<'b>,
        component_map: &'a HashMap<EntityId, Vec<Component>>,
        concept_manager: &'a ConceptManager,
    ) {
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn update_metadata(&mut self, parent: EntityId, same_component_count: u32);
    fn get_parent_entity(&self) -> EntityId;

    fn get_id(&self) -> ComponentId;
}
