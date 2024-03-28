use std::sync::{Arc, Mutex, MutexGuard};

use ecs::scene::Scene;
// use module_manager::ModuleManager;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
    mouse::RelativeMouseState,
    video::Window,
    EventPump, Sdl,
};
use time::{Duration, Instant};

use crate::renderer::Renderer;

pub mod camera;
// pub mod compute;
pub mod model;
pub mod pipeline;
pub mod renderer;
pub mod texture;
pub mod ecs {
    pub mod component;
    pub mod concepts;
    pub mod entity;
    pub mod material;
    pub mod scene;
    pub mod components {
        pub mod camera_component;
        pub mod mesh_component;
        pub mod transform_component;
    }
}

/// Main struct for the engine, manages all higher-level state
///
/// # Example
///
/// ```
/// env_logger::init();
/// let engine = GameZap::builder().build();
/// 'running: loop {
///     for event in engine.event_pump.poll_iter() {
///         match event {
///             Event::Quit { .. } => break 'running,
///
///             Event::Window {
///                 win_event: WindowEvent::Resized(width, height),
///                 ..
///             } => engine
///                 .renderer
///                 .lock()
///                 .unwrap()
///                 .resize((width as u32, height as u32)),
///             _ => {}
///         }
///     }
///     engine.renderer.lock().unwrap().render().unwrap();
/// }
/// ```
pub struct GameZap {
    pub systems: Arc<Mutex<EngineSystems>>,
    pub renderer: Renderer,
    pub clear_color: wgpu::Color,
    pub window: Arc<Window>,
    pub window_size: (u32, u32),
    pub details: Arc<Mutex<EngineDetails>>,

    scenes: Vec<Arc<Mutex<Scene>>>,
    active_scene_index: usize,
}

pub struct EngineDetails {
    pub frame_number: u32,
    pub initialized_instant: Instant,
    pub time_elapsed: Duration,
    pub last_frame_duration: Duration,
    pub time_of_last_frame: Instant,

    pub mouse_state: (Option<RelativeMouseState>, bool),
    pub pressed_scancodes: Vec<Scancode>,
    pub window_aspect_ratio: f32,
}

pub struct EngineSystems {
    pub sdl_context: Arc<Mutex<Sdl>>,
    pub video_subsystem: Arc<Mutex<sdl2::VideoSubsystem>>,
    pub event_pump: Arc<Mutex<sdl2::EventPump>>,
}

pub trait EngineSettings {
    fn update_cursor_mode(&mut self, cursor_visible: bool);
}

impl EngineSettings for Sdl {
    fn update_cursor_mode(&mut self, cursor_visible: bool) {
        self.mouse().show_cursor(cursor_visible);
        self.mouse().set_relative_mouse_mode(!cursor_visible);
    }
}

impl EngineDetails {
    pub fn update_details(
        &mut self,
        event_pump: MutexGuard<EventPump>,
        sdl_context: MutexGuard<Sdl>,
    ) {
        let now = Instant::now();
        self.frame_number += 1;
        self.time_elapsed = now - self.initialized_instant;
        self.last_frame_duration = now - self.time_of_last_frame;
        self.time_of_last_frame = now;

        self.mouse_state = (
            Some(event_pump.relative_mouse_state()),
            sdl_context.mouse().is_cursor_showing(),
        );
        self.pressed_scancodes = event_pump.keyboard_state().pressed_scancodes().collect();
    }
}

impl GameZap {
    /// Initialize certain fields, be sure to call [GameZapBuilder::build()] to build the struct
    pub fn builder() -> GameZapBuilder {
        GameZapBuilder::init()
    }

    pub fn update_details(&self) {
        let mut engine_details = self.details.lock().unwrap();
        let systems = self.systems.lock().unwrap();
        let event_pump = systems.event_pump.lock().unwrap();
        let sdl_context = systems.sdl_context.lock().unwrap();
        engine_details.update_details(event_pump, sdl_context);
    }

    pub fn main_loop(&mut self) {
        'running: loop {
            let active_scene = &mut self.scenes.get(self.active_scene_index);
            {
                let systems = self.systems.lock().unwrap();
                let mut event_pump = systems.event_pump.lock().unwrap();
                for event in event_pump.poll_iter() {
                    if let Some(active_scene_arc) = active_scene {
                        let active_scene = active_scene_arc.lock().unwrap();
                        let component_map = active_scene.get_components();
                        for component in component_map.clone().values().flatten() {
                            component.on_event(
                                &event,
                                &component_map.clone(),
                                &active_scene.get_concept_manager().lock().unwrap(),
                                active_scene.get_active_camera(),
                                &self.details.lock().unwrap(),
                                &systems,
                            );
                        }
                    }
                    // for component in active_scene
                    match event {
                        Event::Quit { .. } => break 'running,
                        Event::Window {
                            win_event: WindowEvent::Resized(width, height),
                            ..
                        } => {
                            self.renderer.resize((width as u32, height as u32));
                            let details_clone = self.details.clone();
                            let mut details = details_clone.lock().unwrap();
                            details.window_aspect_ratio = width as f32 / height as f32;
                        }
                        _ => {}
                    }
                }
            }

            let renderer = &self.renderer;
            if let Some(active_scene_arc) = active_scene {
                let details = self.details.lock().unwrap();
                let mut active_scene = active_scene_arc.lock().unwrap();
                if details.frame_number == 0 {
                    active_scene.initialize(
                        renderer.device.clone(),
                        renderer.queue.clone(),
                        renderer.config.format,
                    );
                }
                drop(details);
                active_scene.update(
                    renderer.device.clone(),
                    renderer.queue.clone(),
                    self.details.clone(),
                    self.systems.clone(),
                );
                active_scene.render(
                    renderer.device.clone(),
                    renderer.queue.clone(),
                    renderer.smaa_target.clone(),
                    renderer.surface.clone(),
                    renderer.depth_texture.clone(),
                    self.window_size,
                );
            }

            self.update_details();
        }
    }
}

/// Builder struct for main [GameZap] struct
pub struct GameZapBuilder {
    sdl_context: Option<sdl2::Sdl>,
    video_subsystem: Option<sdl2::VideoSubsystem>,
    event_pump: Option<sdl2::EventPump>,
    clear_color: wgpu::Color,
    frame_number: u32,
    window: Option<Arc<Window>>,
    window_size: Option<(u32, u32)>,
    initialized_instant: Instant,
    time_elapsed: Duration,
    last_frame_duration: Duration,
    time_of_last_frame: Instant,

    // module_manager: ModuleManager,
    antialiasing: bool,

    scenes: Vec<Arc<Mutex<Scene>>>,
    active_scene_index: usize,
}

impl GameZapBuilder {
    fn init() -> Self {
        GameZapBuilder {
            sdl_context: None,
            video_subsystem: None,
            event_pump: None,
            clear_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            frame_number: 0,
            window: None,
            window_size: None,
            initialized_instant: Instant::now(),
            time_elapsed: Duration::ZERO,
            last_frame_duration: Duration::ZERO,
            time_of_last_frame: Instant::now(),

            // module_manager: ModuleManager::minimal(),
            antialiasing: false,

            scenes: vec![Arc::new(Mutex::new(Scene::default()))],
            active_scene_index: 0,
        }
    }
    /// Pass in a [sdl2::video::Window] object, generates a [Renderer] with a [wgpu::Surface] corresponding to the window
    /// Also specify a [wgpu::Color] clear color (background color for render pass)
    pub fn window_and_renderer(
        mut self,
        sdl_context: sdl2::Sdl,
        video_subsystem: sdl2::VideoSubsystem,
        event_pump: sdl2::EventPump,
        window: Arc<Window>,
        clear_color: wgpu::Color,
    ) -> GameZapBuilder {
        self.window = Some(window.clone());
        self.clear_color = clear_color;
        self.sdl_context = Some(sdl_context);
        self.video_subsystem = Some(video_subsystem);
        self.event_pump = Some(event_pump);
        self.window_size = Some(window.size());
        self
    }

    // pub fn module_manager(mut self, module_manager: ModuleManager) -> GameZapBuilder {
    //     self.module_manager = module_manager;
    //     self
    // }

    pub fn antialiasing(mut self) -> GameZapBuilder {
        self.antialiasing = true;
        self
    }

    pub fn hide_cursor(mut self) -> GameZapBuilder {
        if let Some(sdl_context) = &mut self.sdl_context {
            sdl_context.mouse().show_cursor(false);
            sdl_context.mouse().set_relative_mouse_mode(true);
        }
        self
    }

    pub fn scenes(
        mut self,
        scenes: Vec<Arc<Mutex<Scene>>>,
        active_scene_index: usize,
    ) -> GameZapBuilder {
        self.scenes = scenes;
        self.active_scene_index = active_scene_index;
        self
    }

    /// Pass in a customized [Camera] struct
    /// Default camera uses a 45 degree field of view, starts at (0,0,0),
    /// and points in the positive Z direction

    /// Build the [GameZapBuilder] builder struct into the original [GameZap] struct
    pub fn build(self) -> GameZap {
        let sdl_context = if let Some(context) = self.sdl_context {
            context
        } else {
            sdl2::init().unwrap()
        };
        let video_subsystem = Arc::new(Mutex::new(if let Some(video) = self.video_subsystem {
            video
        } else {
            sdl_context.video().unwrap()
        }));
        let event_pump = Arc::new(Mutex::new(if let Some(pump) = self.event_pump {
            pump
        } else {
            sdl_context.event_pump().unwrap()
        }));

        let window = self.window.unwrap();
        let renderer = pollster::block_on(Renderer::new(
            window.clone(),
            self.clear_color,
            self.antialiasing,
            // self.module_manager,
        ));

        GameZap {
            systems: Arc::new(Mutex::new(EngineSystems {
                sdl_context: Arc::new(Mutex::new(sdl_context)),
                video_subsystem,
                event_pump,
            })),
            renderer,
            clear_color: self.clear_color,
            window,
            window_size: self.window_size.unwrap(),
            details: Arc::new(Mutex::new(EngineDetails {
                frame_number: self.frame_number,
                initialized_instant: self.initialized_instant,
                time_elapsed: self.time_elapsed,
                last_frame_duration: self.last_frame_duration,
                time_of_last_frame: self.time_of_last_frame,
                mouse_state: (None, true),
                pressed_scancodes: vec![],
                window_aspect_ratio: self.window_size.unwrap().0 as f32
                    / self.window_size.unwrap().1 as f32,
            })),
            scenes: self.scenes,
            active_scene_index: self.active_scene_index,
        }
    }
}
