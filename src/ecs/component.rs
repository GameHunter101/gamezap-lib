use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    rc::Rc,
    sync::{Arc, Mutex},
};

use imgui::Ui;
use sdl2::event::Event;
use wgpu::{Device, Queue, RenderPass};

use crate::{ui_manager::UiManager, EngineDetails, EngineSystems};

use super::{concepts::ConceptManager, entity::EntityId, scene::AllComponents};

pub type ComponentId = (EntityId, TypeId, u32);

pub type Component = Box<dyn ComponentSystem>;

#[allow(unused, clippy::too_many_arguments)]
pub trait ComponentSystem: Debug + dyn_clone::DynClone {
    fn register_component(
        &mut self,
        concept_manager: Rc<Mutex<ConceptManager>>,
        data: HashMap<String, Box<dyn Any>>,
    ) {
    }

    fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        component_map: &AllComponents,
        concept_manager: Rc<Mutex<ConceptManager>>,
        engine_details: Option<Rc<Mutex<EngineDetails>>>,
        engine_systems: Option<Rc<Mutex<EngineSystems>>>,
        ui_manager: Rc<Mutex<UiManager>>,
    ) {
    }

    fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        component_map: &mut AllComponents,
        engine_details: Rc<Mutex<EngineDetails>>,
        engine_systems: Rc<Mutex<EngineSystems>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        active_camera_id: Option<EntityId>,
    ) {
    }

    fn ui_draw(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        ui_manager: &mut UiManager,
        ui_frame: &mut Ui,
        component_map: &mut AllComponents,
        concept_manager: Rc<Mutex<ConceptManager>>,
        engine_details: Rc<Mutex<EngineDetails>>,
        engine_systems: Rc<Mutex<EngineSystems>>,
    ) {
    }

    fn render<'a: 'b, 'b>(
        &'a self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        render_pass: &mut RenderPass<'b>,
        component_map: &'a HashMap<EntityId, Vec<Component>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        engine_details: &EngineDetails,
        engine_systems: &EngineSystems,
    ) {
    }

    fn on_event(
        &self,
        event: &Event,
        component_map: &HashMap<EntityId, Vec<Component>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        active_camera_id: Option<EntityId>,
        engine_details: &EngineDetails,
        engine_systems: &EngineSystems,
    ) {
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn update_metadata(&mut self, parent: EntityId, same_component_count: u32);
    fn get_parent_entity(&self) -> EntityId;

    fn get_id(&self) -> ComponentId;

    fn render_order(&self) -> usize {
        0
    }
}
