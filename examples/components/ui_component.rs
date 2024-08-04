use std::fmt::Debug;

use gamezap::{new_component, texture::Texture, ui_manager::UiManager};
use imgui::Ui;

new_component!(
    UiComponent {
        font_path: String,
        font_id: Option<imgui::FontId>,
        image_details: Option<(imgui::TextureId, [f32; 2])>,
        picture_enabled: bool
    }
);

impl UiComponent {
    pub fn new(font_path: &str) -> UiComponent {
        UiComponent {
            parent: 0,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
            font_path: font_path.to_string(),
            font_id: None,
            image_details: None,
            picture_enabled: false,
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

    fn update(
        &mut self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &mut AllComponents,
        _engine_details: Rc<Mutex<EngineDetails>>,
        engine_systems: Rc<Mutex<EngineSystems>>,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
        entities: &mut Vec<Entity>,
        _materials: Option<&(Vec<Material>, usize)>,
        _compute_pipelines: &[ComputePipeline],
    ) {
        if engine_systems
            .lock()
            .unwrap()
            .sdl_context
            .mouse()
            .is_cursor_showing()
            != self.picture_enabled
        {
            entities[0].enabled = !self.picture_enabled;
        }
        self.picture_enabled = !self.picture_enabled;
    }

    fn ui_draw(
        &mut self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
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
            ui_frame
                .window(".")
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
}
