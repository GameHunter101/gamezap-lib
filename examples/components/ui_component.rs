use std::{
    any::{Any, TypeId},
    fmt::Debug,
    rc::Rc,
    sync::{Arc, Mutex},
};

use gamezap::{texture::Texture, ui_manager::UiManager};
use imgui::Ui;
use wgpu::{Device, Queue};

use crate::{
    gamezap::ecs::{
        component::{ComponentId, ComponentSystem},
        scene::AllComponents,
    },
    EngineDetails, EngineSystems,
};

use super::super::{gamezap::ecs::concepts::ConceptManager, gamezap::ecs::entity::EntityId};

#[derive(Debug, Clone)]
pub struct UiComponent {
    parent: EntityId,
    id: ComponentId,
    font_path: String,
    font_id: Option<imgui::FontId>,
    image_details: Option<(imgui::TextureId, [f32; 2])>,
}

impl UiComponent {
    pub fn new(font_path: &str) -> UiComponent {
        UiComponent {
            parent: 0,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            font_path: font_path.to_string(),
            font_id: None,
            image_details: None,
        }
    }
}

impl ComponentSystem for UiComponent {
    fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        _component_map: &AllComponents,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: Option<Rc<Mutex<EngineDetails>>>,
        _engine_systems: Option<Rc<Mutex<EngineSystems>>>,
        ui_manager: Rc<Mutex<UiManager>>,
    ) {
        let mut ui_manager = ui_manager.lock().unwrap();
        self.font_id = Some(
            ui_manager
                .load_font("Inter", self.font_path.clone(), 20.0)
                .unwrap(),
        );
        let mut renderer = ui_manager.imgui_renderer.lock().unwrap();
        let details = Texture::load_ui_image(&device, &queue,
            &mut renderer, "C:\\Users\\liors\\Documents\\Coding projects\\Rust\\gamezap-lib\\assets\\testing_textures\\dude.png".to_string());
        self.image_details = Some(details);
    }

    fn ui_draw(
        &mut self,
        _ui_manager: &mut UiManager,
        ui_frame: &mut Ui,
        _component_map: &mut AllComponents,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        engine_details: Rc<Mutex<EngineDetails>>,
        engine_systems: Rc<Mutex<EngineSystems>>,
    ) {
        if engine_systems
            .lock()
            .unwrap()
            .sdl_context
            .mouse()
            .is_cursor_showing()
        {
            let _inter = ui_frame.push_font(self.font_id.unwrap());
            ui_frame.window(".")
                .title_bar(false)
                .draw_background(false)
                .resizable(false)
                .movable(false)
                .always_auto_resize(true)
                .position([100.0, 100.0], imgui::Condition::Always)
                .build(|| {
                    ui_frame.text(format!("FPS: {}", engine_details.lock().unwrap().fps,));
                    ui_frame.text(format!(
                        "Frame time (us): {}",
                        engine_details
                            .lock()
                            .unwrap()
                            .last_frame_duration
                            .as_micros()
                    ));
                    imgui::Image::new(self.image_details.unwrap().0, self.image_details.unwrap().1)
                        .build(ui_frame);
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
