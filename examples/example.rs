use std::sync::Arc;

use gamezap::{GameZap, ecs::scene::Scene};
use nalgebra as na;

extern crate gamezap;

#[tokio::main]
async fn main() {
    env_logger::init();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    let application_title = "Test";
    let window_size = (800, 600);
    let window = Arc::new(
        video_subsystem
            .window(application_title, window_size.0, window_size.1)
            .resizable()
            .build()
            .unwrap(),
    );

    let mut scene = Scene::new();

    let scenes = vec![scene];

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
        .antialiasing()
        // .hide_cursor()
        .scenes(scenes, 0)
        .build();

    engine.main_loop();
}
