use std::fmt::Debug;

use gamezap::{new_component, texture::Texture, ui_manager::UiManager, ecs::scene::TextParams};
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
        _text_items: &mut Vec<TextParams>,
    ) {
        let mut ui_manager = ui_manager.lock().unwrap();
        self.font_id = Some(
            ui_manager
                .load_font("Inter", self.font_path.clone(), 20.0)
                .unwrap(),
        );
        let mut renderer = ui_manager.imgui_renderer.lock().unwrap();
        let details = Texture::load_ui_image(
            &device,
            &queue,
            &mut renderer,
            "assets\\testing_textures\\dude.png".to_string(),
        );
        self.image_details = Some(details);
    }

    fn update(
        &mut self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &mut AllComponents,
        _engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
        concept_manager: Rc<Mutex<ConceptManager>>,
        active_camera_id: Option<EntityId>,
        entities: &mut Vec<Entity>,
        _materials: Option<&mut (Vec<Material>, usize)>,
        _compute_pipelines: &mut [ComputePipeline],
        _text_items: &mut Vec<TextParams>,
    ) {
        // println!("cursor: {}", engine_systems.lock().unwrap().sdl_context.mouse().is_cursor_showing());
        let concept_manager = concept_manager.lock().unwrap();
        let is_cursor_visible = *concept_manager
            .get_concept::<bool>(
                (
                    active_camera_id.unwrap(),
                    TypeId::of::<super::keyboard_input_component::KeyboardInputComponent>(),
                    0,
                ),
                "is_cursor_visible".to_string(),
            )
            .unwrap();
        entities[self.id.0 as usize].enabled = is_cursor_visible;
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
