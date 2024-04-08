use std::{
    any::{Any, TypeId},
    fmt::Debug,
    sync::{Arc, Mutex},
};

use wgpu::{Device, Queue};

use crate::{
    ecs::{
        component::{ComponentId, ComponentSystem},
        scene::AllComponents,
    },
    EngineDetails, EngineSystems,
};

use super::super::{concepts::ConceptManager, entity::EntityId};

#[derive(Debug, Clone)]
pub struct UiComponent {
    parent: EntityId,
    id: ComponentId,
}

impl UiComponent {
    pub fn new() -> UiComponent {
        UiComponent {
            parent: 0,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
        }
    }
}

impl Default for UiComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentSystem for UiComponent {
    fn update(
        &mut self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: AllComponents,
        engine_details: Arc<Mutex<EngineDetails>>,
        engine_systems: Arc<Mutex<EngineSystems>>,
        _concept_manager: Arc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
    ) {
        let systems = engine_systems.lock().unwrap();
        let ui_manager = systems.ui_manager.lock().unwrap();
        let mut imgui_context = ui_manager.imgui_context.lock().unwrap();

        let ui = imgui_context.new_frame();

        ui.window("Hello World")
            .size([300.0, 100.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text("Heyo");
                ui.separator();
                let moues_pos = ui.io().mouse_pos;
                ui.text(format!(
                    "Mouse position: ({:.1}, {:.1})",
                    moues_pos[0], moues_pos[1]
                ));
                let details = engine_details.lock().unwrap();
                ui.text(format!("Frame time: {}", details.last_frame_duration.whole_milliseconds()))
            });

        ui.show_demo_window(&mut true);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn update_metadata(&mut self, parent: EntityId, same_component_count: u32) {
        self.parent = parent;
        self.id.0 = parent;
        self.id.2 = same_component_count;
    }

    fn get_parent_entity(&self) -> EntityId {
        self.parent
    }

    fn get_id(&self) -> ComponentId {
        self.id
    }
}
