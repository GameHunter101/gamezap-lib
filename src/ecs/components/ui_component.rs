use std::{
    any::{Any, TypeId},
    fmt::Debug,
    rc::Rc,
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
    font_path: String,
    font_id: Option<imgui::FontId>,
}

impl UiComponent {
    pub fn new(font_path: &str) -> UiComponent {
        UiComponent {
            parent: 0,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            font_path: font_path.to_string(),
            font_id: None,
        }
    }
}

impl ComponentSystem for UiComponent {
    fn initialize(
        &mut self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &AllComponents,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: Option<Rc<Mutex<EngineDetails>>>,
        engine_systems: Option<Rc<Mutex<EngineSystems>>>,
    ) {
        let systems_arc = engine_systems.unwrap();
        let systems = systems_arc.lock().unwrap();
        let mut ui_manager = systems.ui_manager.lock().unwrap();
        self.font_id = Some(
            ui_manager
                .load_font("Inter", self.font_path.clone(), 12.0)
                .unwrap(),
        );
    }

    fn update(
        &mut self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &AllComponents,
        engine_details: Rc<Mutex<EngineDetails>>,
        engine_systems: Rc<Mutex<EngineSystems>>,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
    ) {
        if engine_systems
            .lock()
            .unwrap()
            .sdl_context
            .mouse()
            .is_cursor_showing()
        {
            let systems = engine_systems.lock().unwrap();
            let mut ui_manager = systems.ui_manager.lock().unwrap();
            ui_manager.set_render_flag();

            let mut imgui_context = ui_manager.imgui_context.lock().unwrap();

            let ui = imgui_context.new_frame();

            /* ui.window("Hello World")
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
                ui.text(format!(
                    "Frame time: {}",
                    details.last_frame_duration.whole_milliseconds()
                ));
                ui.text(format!("FPS: {}", details.fps,));
            }); */
            let _inter = ui.push_font(self.font_id.unwrap());
            ui.window(".")
                .title_bar(false)
                .draw_background(false)
                .resizable(false)
                .movable(false)
                .always_auto_resize(true)
                .position([100.0, 100.0], imgui::Condition::Always)
                .build(|| {
                    ui.text(format!("FPS: {}", engine_details.lock().unwrap().fps,));
                    ui.text(format!(
                        "Frame time (μs): {}",
                        engine_details
                            .lock()
                            .unwrap()
                            .last_frame_duration
                            .as_micros()
                    ));
                });
            _inter.pop();

            // ui.show_demo_window(&mut true);
        }
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
