use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use camera::Camera;
use sdl2::video::Window;
use time::{Duration, Instant};

use crate::renderer::Renderer;

pub mod camera;
pub mod light;
pub mod model;
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
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub event_pump: sdl2::EventPump,
    pub renderer: Arc<Mutex<Renderer>>,
    pub clear_color: wgpu::Color,
    pub camera: Camera,
    pub frame_number: u32,
    pub window: Rc<Window>,
    pub window_size: (u32, u32),
    pub initialized_instant: time::Instant,
    pub time_elapsed: time::Duration,
    pub last_frame_time: time::Duration,
}

impl GameZap {
    /// Initialize certain fields, be sure to call [GameZapBuilder::build()] to build the struct
    pub fn builder() -> GameZapBuilder {
        GameZapBuilder::init()
    }
}

/// Builder struct for main [GameZap] struct
pub struct GameZapBuilder {
    sdl_context: Option<sdl2::Sdl>,
    video_subsystem: Option<sdl2::VideoSubsystem>,
    event_pump: Option<sdl2::EventPump>,
    clear_color: wgpu::Color,
    camera: Camera,
    frame_number: u32,
    window: Option<Rc<Window>>,
    window_size: Option<(u32, u32)>,
    initialized_instant: time::Instant,
    time_elapsed: time::Duration,
    last_frame_time: time::Duration,
}

impl GameZapBuilder {
    pub fn init() -> Self {
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
            camera: Camera::default(),
            frame_number: 0,
            window: None,
            window_size: None,
            initialized_instant: Instant::now(),
            time_elapsed: Duration::ZERO,
            last_frame_time: Duration::ZERO,
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

    /// Pass in a customized [Camera] struct
    /// Default camera uses a 45 degree field of view, starts at (0,0,0),
    /// and points in the positive Z direction
    pub fn camera(mut self, camera: Camera) -> GameZapBuilder {
        self.camera = camera;
        self
    }

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
        let renderer = Arc::new(Mutex::new(pollster::block_on(Renderer::new(
            window.clone(),
            self.clear_color,
        ))));

        GameZap {
            sdl_context,
            video_subsystem,
            event_pump,
            renderer: renderer.clone(),
            clear_color: self.clear_color,
            camera: self.camera.create_descriptor_and_buffer(&renderer.clone().lock().unwrap().device),
            frame_number: self.frame_number,
            window: window,
            window_size: self.window_size.unwrap(),
            initialized_instant: self.initialized_instant,
            time_elapsed: self.time_elapsed,
            last_frame_time: self.last_frame_time,
        }
    }
}
