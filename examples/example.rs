use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use gamezap::{camera::Camera, renderer::Renderer, GameZap};
use sdl2::event::{Event, WindowEvent};

extern crate gamezap;
fn main() {
    env_logger::init();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    let application_title = "Test";
    let window_size = (800, 600);
    let window = Rc::new(
        video_subsystem
            .window(application_title, window_size.0, window_size.1)
            .resizable()
            .build()
            .unwrap(),
    );

    let mut engine = GameZap::builder()
        .window_and_renderer(
            sdl_context,
            video_subsystem,
            event_pump,
            window,
            wgpu::Color {
                r: 0.2,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        )
        .build();
    'running: loop {
        for event in engine.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => engine
                    .renderer
                    .lock()
                    .unwrap()
                    .resize((width as u32, height as u32)),
                _ => {}
            }
        }
        engine.renderer.lock().unwrap().render().unwrap();
    }
}
