use ecs::scene::Scene;
use engine_management::rendering_management::RenderingManager;
use std::fmt::Debug;

use winit::{
    event::Event::WindowEvent,
    event_loop::EventLoop,
    window::{Fullscreen, Window, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;

pub mod engine_management {
    pub mod rendering_management;
}

pub mod engine_support {
    pub mod texture_support;
}

pub mod ecs {
    pub mod actions;
    pub mod component;
    pub mod entity;
    pub mod material;
    pub mod pipeline;
    pub mod scene;
    pub mod builtin_actions {
        pub mod workload_action;
    }
}

/// The main engine struct. Contains the state for the whole engine.
pub struct Gamezap<'a> {
    event_loop: EventLoop<()>,
    input_manager: WinitInputHelper,
    rendering_manager: RenderingManager,
    scenes: Vec<Scene<'a>>,
    active_scene: usize,
    initialized_scene: bool,
    window: Window,
}

impl<'a> Gamezap<'a> {
    pub fn builder() -> GamezapBuilder {
        GamezapBuilder::default()
    }

    pub async fn main_loop(mut self) {
        let mut last_active_scene_index = self.active_scene;
        self.event_loop
            .run(move |event, elwt| match &event {
                WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::Resized(new_size) => {
                        self.rendering_manager
                            .resize(new_size.width, new_size.height);
                    }
                    winit::event::WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    _ => {}
                },
                winit::event::Event::AboutToWait => {
                    if self.scenes.is_empty() {
                        return;
                    }
                    if !self.initialized_scene {
                        self.scenes[self.active_scene].initialize();
                        self.initialized_scene = true;
                    }
                    if self.active_scene != last_active_scene_index {
                        self.initialized_scene = false;
                        last_active_scene_index = self.active_scene;
                        return;
                    }
                    let scene = &mut self.scenes[self.active_scene];
                    self.rendering_manager.render(scene);
                    let device = self.rendering_manager.get_device();
                    let queue = self.rendering_manager.get_queue();
                    // let smaa_target = self.rendering_manager.smaa_target_mut();
                    // scene.render(device, queue, output, smaa_target);
                    async_scoped::TokioScope::scope_and_block(|scope| {
                        let proc = async {
                            scene.update(device, queue, &self.input_manager).await;
                        };
                        scope.spawn(proc);
                    });
                }
                _ => {}
            })
            .expect("An error occured in the main loop.");
    }

    pub fn attach_scene(&mut self, scene: Scene<'a>) -> usize {
        let index = self.scenes.len();
        self.scenes.push(scene);

        index
    }

    pub fn rendering_manager(&self) -> &RenderingManager {
        &self.rendering_manager
    }
}

impl<'a> Debug for Gamezap<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowAndEventManager")
            .field("event_loop", &self.event_loop)
            .field("window", &self.window)
            .field("input_manager", &"Input Manager")
            .finish()
    }
}

#[derive(Debug)]
pub struct GamezapBuilder {
    width: u32,
    height: u32,
    title: &'static str,
    fullscreen: Option<Fullscreen>,
    antialiasing_enabled: bool,
    clear_color: wgpu::Color,
}

impl Default for GamezapBuilder {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            title: "GameZap Program",
            fullscreen: None,
            antialiasing_enabled: false,
            clear_color: wgpu::Color::BLACK,
        }
    }
}

impl<'a> GamezapBuilder {
    pub fn window_settings(
        mut self,
        width: u32,
        height: u32,
        title: &'static str,
        fullscreen: Option<Fullscreen>,
    ) -> Self {
        self.width = width;
        self.height = height;
        self.title = title;
        self.fullscreen = fullscreen;
        self
    }

    pub fn antialiasing_enabled(mut self, enabled: bool) -> Self {
        self.antialiasing_enabled = enabled;
        self
    }

    pub fn clear_color(mut self, color: wgpu::Color) -> Self {
        self.clear_color = color;
        self
    }

    pub async fn build(self) -> Gamezap<'a> {
        let event_loop = EventLoop::new().expect("Failed to create event loop.");
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let window = WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height))
            .with_title(self.title)
            .with_fullscreen(self.fullscreen)
            .build(&event_loop)
            .expect("Failed to create window.");
        let input_manager = WinitInputHelper::new();

        let rendering_manager =
            RenderingManager::new(&window, self.antialiasing_enabled, self.clear_color).await;

        Gamezap {
            rendering_manager,
            event_loop,
            input_manager,
            scenes: Vec::new(),
            active_scene: 0,
            initialized_scene: false,
            window,
        }
    }
}
