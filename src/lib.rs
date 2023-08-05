use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use camera::Camera;
use sdl2::{
    event::{Event, WindowEvent},
    video::Window,
};

use crate::renderer::Renderer;

mod camera;
mod light;
mod model;
mod pipeline;
mod renderer;
mod texture;

/// Main struct for the engine, manages all higher-level state
///
/// # Example
///
/// ```
/// env_logger::init();
/// let engine = GameZap::builder().build();
/// ```
pub struct GameZap<'a> {
    sdl_context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    application_title: &'static str,
    pub renderer: Arc<Mutex<Renderer>>,
    camera: Camera,
    event_loop: Box<dyn Fn(&'a mut sdl2::EventPump) -> ()>,
    frame_number: u32,
    window: Rc<Window>,
    window_size: (u32, u32),
    initialized_instant: time::Instant,
    time_elapsed: time::Duration,
    last_frame_time: time::Duration,
}

impl<'a> GameZap<'a> {
    /// Initialize certain fields, be sure to call [GameZapBuilde::build()] to build the struct
    pub fn builder() -> GameZapBuilder<'a> {
        GameZapBuilder::default()
    }

    /// The main game loop, handle events and render calls here
    pub fn main_loop(&self) {
        let sdl_context = sdl2::init().unwrap();
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        // let event_pump_ref::EventPump = &mut event_pump;
        // {
        //     (self.event_loop)(event_pump_ref);
        // }
    }
}

/// Builder struct for main [GameZap] struct
pub struct GameZapBuilder<'a> {
    sdl_context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    application_title: &'static str,
    renderer: Arc<Mutex<Renderer>>,
    camera: Camera,
    event_loop: Box<dyn Fn(&'a mut sdl2::EventPump) -> ()>,
    frame_number: u32,
    window: Rc<Window>,
    window_size: (u32, u32),
    initialized_instant: time::Instant,
    time_elapsed: time::Duration,
    last_frame_time: time::Duration,
}

impl<'a> GameZapBuilder<'a> {
    /// Pass in a [sdl2::video::Window] object, generates a [Renderer] with a [wgpu::Surface] corresponding to the window
    /// Also specify a [wgpu::Color] clear color (background color for render pass)
    pub fn window_and_renderer(
        mut self,
        application_name: &'static str,
        window: Window,
        clear_color: wgpu::Color,
    ) -> GameZapBuilder<'a> {
        let window_rc = Rc::new(window);
        self.window = window_rc.clone();
        self.renderer = Arc::new(Mutex::new(pollster::block_on(Renderer::new(
            window_rc,
            clear_color,
        ))));
        self
    }

    /// Pass in a function or closure that uses a [sdl2::EventPump]
    /// This function should return false when you want to close the program
    ///
    /// # Example
    ///
    /// ```
    /// GameZap::builder().input_handler(|event_pump| {
    ///     for event in event_pump.poll_iter() {
    ///         match event {
    ///             Event::Quit { .. } => return false,
    ///             Event::Window {
    ///                 win_event: WindowEvent::Resized(width, height),
    ///                 ..
    ///             } => renderer.resize((width as u32, height as u32)),
    ///             _ => {}
    ///         }
    ///     }
    ///     ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    /// })
    /// ```
    pub fn event_loop(
        mut self,
        event_loop: Box<dyn Fn(&'a mut sdl2::EventPump) -> ()>,
    ) -> GameZapBuilder<'a> {
        self.event_loop = event_loop;
        self
    }

    pub fn camera(mut self, camera: Camera) -> GameZapBuilder<'a> {
        self.camera = camera;
        self
    }

    pub fn build(self) -> GameZap<'a> {
        GameZap {
            sdl_context: self.sdl_context,
            video_subsystem: self.video_subsystem,
            application_title: self.application_title,
            renderer: self.renderer,
            camera: self.camera,
            event_loop: self.event_loop,
            frame_number: self.frame_number,
            window: self.window,
            window_size: self.window_size,
            initialized_instant: self.initialized_instant,
            time_elapsed: self.time_elapsed,
            last_frame_time: self.last_frame_time,
        }
    }
}

impl<'a> std::default::Default for GameZapBuilder<'a> {
    fn default() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let application_title = "Application window";
        let window_size = (800, 600);
        let window = Rc::new(
            video_subsystem
                .window(application_title, window_size.0, window_size.1)
                .resizable()
                .build()
                .unwrap(),
        );
        let renderer = Arc::new(Mutex::new(pollster::block_on(Renderer::new(
            window.clone(),
            wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
        ))));
        GameZapBuilder {
            sdl_context,
            video_subsystem,
            application_title,
            renderer: renderer.clone(),
            camera: Camera::default(),
            event_loop: Box::new(move |event_pump| {
                let renderer = renderer.clone();
                'running: loop {
                    for event in event_pump.poll_iter() {
                        match event {
                            Event::Quit { .. } => break 'running,
                            Event::Window {
                                win_event: WindowEvent::Resized(width, height),
                                ..
                            } => renderer
                                .lock()
                                .unwrap()
                                .resize((width as u32, height as u32)),
                            _ => {}
                        }
                    }
                }
            }),
            frame_number: 0,
            window: window.clone(),
            window_size,
            initialized_instant: time::Instant::now(),
            time_elapsed: time::Duration::ZERO,
            last_frame_time: time::Duration::ZERO,
        }
    }
}
