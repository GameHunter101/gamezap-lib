use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use module_manager::ModuleManager;
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
pub mod light;
pub mod materials;
pub mod model;
pub mod module_manager;
pub mod pipeline;
pub mod renderer;
pub mod texture;

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
    pub sdl_context: Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub event_pump: sdl2::EventPump,
    pub renderer: RefCell<Renderer>,
    pub clear_color: wgpu::Color,
    pub window: Rc<Window>,
    pub window_size: (u32, u32),
    pub details: RefCell<EngineDetails>,
}

pub struct EngineDetails {
    pub frame_number: u32,
    pub initialized_instant: time::Instant,
    pub time_elapsed: time::Duration,
    pub last_frame_duration: time::Duration,
    pub time_of_last_frame: time::Instant,

    pub mouse_state: (Option<RelativeMouseState>, bool),
    pub pressed_scancodes: Vec<Scancode>,
}

impl EngineDetails {
    pub fn update_details(&mut self, event_pump: &EventPump, sdl_context: &Sdl) {
        let now = time::Instant::now();
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
        let mut engine_details = self.details.borrow_mut();
        engine_details.update_details(&self.event_pump, &self.sdl_context);
    }

    pub fn update_renderer(&self) -> RefMut<'_, Renderer> {
        let renderer = self.renderer.borrow_mut();
        renderer.update_buffers();
        renderer.render().unwrap();
        renderer
    }

    pub fn main_loop(
        &mut self,
        extensions: Vec<Box<dyn Fn(RefMut<EngineDetails>, RefMut<Renderer>)>>,
    ) {
        'running: loop {
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => self
                        .renderer
                        .borrow_mut()
                        .resize((width as u32, height as u32)),
                    _ => {}
                }
            }
            for func in &extensions {
                (func)(self.details.borrow_mut(), self.renderer.borrow_mut());
            }
            self.update_renderer();
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
    window: Option<Rc<Window>>,
    window_size: Option<(u32, u32)>,
    initialized_instant: time::Instant,
    time_elapsed: time::Duration,
    last_frame_duration: time::Duration,
    time_of_last_frame: time::Instant,

    module_manager: ModuleManager,
    antialiasing: bool,
}

impl<'a> GameZapBuilder {
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

            module_manager: ModuleManager::minimal(),
            antialiasing: false,
        }
    }
    /// Pass in a [sdl2::video::Window] object, generates a [Renderer] with a [wgpu::Surface] corresponding to the window
    /// Also specify a [wgpu::Color] clear color (background color for render pass)
    pub fn window_and_renderer(
        mut self,
        sdl_context: sdl2::Sdl,
        video_subsystem: sdl2::VideoSubsystem,
        event_pump: sdl2::EventPump,
        window: Rc<Window>,
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

    pub fn module_manager(mut self, module_manager: ModuleManager) -> GameZapBuilder {
        self.module_manager = module_manager;
        self
    }

    pub fn antialiasing(mut self) -> GameZapBuilder {
        self.antialiasing = true;
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
        let video_subsystem = if let Some(video) = self.video_subsystem {
            video
        } else {
            sdl_context.video().unwrap()
        };
        let event_pump = if let Some(pump) = self.event_pump {
            pump
        } else {
            sdl_context.event_pump().unwrap()
        };

        let window = self.window.unwrap();
        let renderer = RefCell::new(pollster::block_on(Renderer::new(
            window.clone(),
            self.clear_color,
            self.antialiasing,
            self.module_manager,
        )));

        GameZap {
            sdl_context,
            video_subsystem,
            event_pump,
            renderer,
            clear_color: self.clear_color,
            window: window,
            window_size: self.window_size.unwrap(),
            details: RefCell::new(EngineDetails {
                frame_number: self.frame_number,
                initialized_instant: self.initialized_instant,
                time_elapsed: self.time_elapsed,
                last_frame_duration: self.last_frame_duration,
                time_of_last_frame: self.time_of_last_frame,
                mouse_state: (None, true),
                pressed_scancodes: vec![],
            }),
        }
    }
}
