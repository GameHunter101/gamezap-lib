use std::{
    cell::{Ref, RefCell},
    rc::Rc,
    sync::Mutex,
    time::{Duration, Instant},
};

use ecs::scene::Scene;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
    mouse::RelativeMouseState,
    video::Window,
    EventPump, Sdl, VideoSubsystem,
};
use ui_manager::UiManager;

use crate::renderer::Renderer;

// pub mod compute;
pub mod model;
pub mod pipeline;
pub mod renderer;
pub mod texture;
pub mod ui_manager;
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
        pub mod ui_component;
    }
}

pub struct GameZap {
    pub systems: Rc<Mutex<EngineSystems>>,
    pub renderer: Renderer,
    pub clear_color: wgpu::Color,
    pub window: Window,
    pub window_size: (u32, u32),
    pub details: Rc<Mutex<EngineDetails>>,

    scenes: Vec<Scene>,
    active_scene_index: usize,
}

pub struct EngineDetails {
    pub frame_number: u128,
    pub initialized_instant: Instant,
    pub time_elapsed: Duration,
    pub last_frame_duration: Duration,
    pub time_of_last_frame: Instant,
    time_of_last_fps_calc: Instant,
    frame_count_at_last_fps_calc: u128,
    pub fps: u32,

    pub mouse_state: (Option<RelativeMouseState>, bool),
    pub pressed_scancodes: Vec<Scancode>,
    pub window_aspect_ratio: f32,
    pub render_mask: Option<RenderMask>,
}

pub struct EngineSystems {
    pub sdl_context: Sdl,
    pub video_subsystem: VideoSubsystem,
    pub event_pump: RefCell<EventPump>,
    pub ui_manager: Rc<Mutex<UiManager>>,
}

pub struct RenderMask {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
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
    pub fn update_details(&mut self, event_pump: Ref<EventPump>, sdl_context: &Sdl) {
        let now = Instant::now();
        self.frame_number += 1;
        self.time_elapsed = now - self.initialized_instant;
        self.last_frame_duration = now - self.time_of_last_frame;
        self.time_of_last_frame = now;

        if (now - self.time_of_last_fps_calc).as_secs_f32() > 1.0 {
            self.time_of_last_fps_calc = now;
            self.fps = (self.frame_number - self.frame_count_at_last_fps_calc) as u32;
            self.frame_count_at_last_fps_calc = self.frame_number;
        }

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

    pub fn update_details(&mut self) {
        let mut details = self.details.lock().unwrap();
        let systems = self.systems.lock().unwrap();
        details.update_details(systems.event_pump.borrow(), &systems.sdl_context);
    }

    pub fn main_loop(&mut self) {
        'running: loop {
            let active_scene_opt = self.scenes.get_mut(self.active_scene_index);
            {
                let systems = self.systems.lock().unwrap();

                let mut event_pump = systems.event_pump.borrow_mut();

                let ui_manager = systems.ui_manager.lock().unwrap();
                let mut imgui_context = ui_manager.imgui_context.lock().unwrap();
                let mut imgui_platform = ui_manager.imgui_platform.lock().unwrap();

                for event in event_pump.poll_iter() {
                    imgui_platform.handle_event(&mut imgui_context, &event);
                    if imgui_platform.ignore_event(&event) {
                        continue;
                    }

                    if let Some(active_scene) = &active_scene_opt {
                        let component_map = active_scene.get_components();
                        for component in component_map.values().flatten() {
                            component.on_event(
                                &event,
                                component_map,
                                active_scene.get_concept_manager(),
                                active_scene.get_active_camera(),
                                &self.details.lock().unwrap(),
                                &systems,
                            );
                        }
                    }

                    match event {
                        Event::Quit { .. } => break 'running,
                        Event::Window {
                            win_event: WindowEvent::Resized(width, height),
                            ..
                        } => {
                            self.renderer.resize((width as u32, height as u32));
                            self.details.lock().unwrap().window_aspect_ratio =
                                width as f32 / height as f32;
                        }
                        _ => {}
                    }
                }

                imgui_platform.prepare_frame(
                    imgui_context.io_mut(),
                    &self.window,
                    &event_pump.mouse_state(),
                );
            }

            {
                let renderer = &self.renderer;

                let output = renderer.surface.get_current_texture().unwrap();
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut smaa_binding = renderer.smaa_target.lock().unwrap();
                let smaa_frame = smaa_binding.start_frame(&renderer.device, &renderer.queue, &view);

                if let Some(active_scene) = active_scene_opt {
                    if self.details.lock().unwrap().frame_number == 0 {
                        active_scene.initialize(
                            renderer.device.clone(),
                            renderer.queue.clone(),
                            renderer.config.format,
                            self.details.clone(),
                            self.systems.clone(),
                        );
                        let systems = self.systems.lock().unwrap();
                        let ui_manager = systems.ui_manager.lock().unwrap();
                        let mut imgui_renderer = ui_manager.imgui_renderer.lock().unwrap();
                        let mut context = ui_manager.imgui_context.lock().unwrap();
                        imgui_renderer.reload_font_texture(
                            &mut context,
                            &renderer.device.clone(),
                            &renderer.queue.clone(),
                        );
                    }
                    active_scene.update(
                        renderer.device.clone(),
                        renderer.queue.clone(),
                        self.details.clone(),
                        self.systems.clone(),
                    );
                    active_scene.render(
                        renderer.device.clone(),
                        renderer.queue.clone(),
                        renderer.depth_texture.clone(),
                        self.window_size,
                        &self.details.lock().unwrap(),
                        &self.systems.lock().unwrap(),
                        smaa_frame,
                        output,
                    );
                }
            }

            self.update_details();
        }
    }

    pub fn create_scene(&mut self, scene: Scene) {
        self.scenes.push(scene);
    }
}

/// Builder struct for main [GameZap] struct
#[allow(dead_code)]
pub struct GameZapBuilder {
    sdl_context: Option<sdl2::Sdl>,
    video_subsystem: Option<VideoSubsystem>,
    event_pump: Option<EventPump>,
    clear_color: wgpu::Color,
    frame_number: u128,
    window: Option<Window>,
    window_size: Option<(u32, u32)>,
    initialized_instant: Instant,
    time_elapsed: Duration,
    last_frame_duration: Duration,
    time_of_last_frame: Instant,

    antialiasing: bool,

    active_scene_index: usize,

    render_mask: Option<RenderMask>,
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

            antialiasing: false,

            active_scene_index: 0,

            render_mask: None,
        }
    }
    /// Pass in a [sdl2::video::Window] object, generates a [Renderer] with a [wgpu::Surface] corresponding to the window
    /// Also specify a [wgpu::Color] clear color (background color for render pass)
    pub fn window_and_renderer(
        mut self,
        sdl_context: sdl2::Sdl,
        video_subsystem: VideoSubsystem,
        event_pump: EventPump,
        window: Window,
        clear_color: wgpu::Color,
    ) -> GameZapBuilder {
        self.window_size = Some(window.size());
        self.window = Some(window);
        self.clear_color = clear_color;
        self.sdl_context = Some(sdl_context);
        self.video_subsystem = Some(video_subsystem);
        self.event_pump = Some(event_pump);
        self
    }

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

    pub fn render_mask(mut self, mask: RenderMask) -> GameZapBuilder {
        self.render_mask = Some(mask);
        self
    }

    /// Build the [GameZapBuilder] builder struct into the original [GameZap] struct
    pub async fn build(self) -> GameZap {
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
        let event_pump = RefCell::new(if let Some(pump) = self.event_pump {
            pump
        } else {
            sdl_context.event_pump().unwrap()
        });

        let window = self.window.unwrap();

        let renderer = Renderer::new(&window, self.clear_color, self.antialiasing).await;

        let ui_manager = Rc::new(Mutex::new(UiManager::new(
            renderer.surface_format,
            renderer.device.clone(),
            renderer.queue.clone(),
            &window,
        )));

        GameZap {
            systems: Rc::new(Mutex::new(EngineSystems {
                sdl_context,
                video_subsystem,
                event_pump,
                ui_manager,
            })),
            renderer,
            clear_color: self.clear_color,
            window,
            window_size: self.window_size.unwrap(),
            details: Rc::new(Mutex::new(EngineDetails {
                frame_number: self.frame_number,
                initialized_instant: self.initialized_instant,
                time_elapsed: self.time_elapsed,
                last_frame_duration: self.last_frame_duration,
                time_of_last_frame: self.time_of_last_frame,
                time_of_last_fps_calc: self.initialized_instant,
                frame_count_at_last_fps_calc: self.frame_number,
                fps: 0,

                mouse_state: (None, true),
                pressed_scancodes: vec![],
                window_aspect_ratio: self.window_size.unwrap().0 as f32
                    / self.window_size.unwrap().1 as f32,
                render_mask: self.render_mask,
            })),
            scenes: Vec::new(),
            active_scene_index: self.active_scene_index,
        }
    }
}
