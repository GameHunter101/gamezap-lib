use std::sync::{Arc, Mutex};

use gamezap::{
    ecs::{
        component::{CameraComponent, Material, MeshComponent},
        scene::Scene,
    },
    model::Vertex,
    GameZap,
};
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

    let scene = Arc::new(Mutex::new(Scene::new()));


    let scenes = vec![scene.clone()];

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

    let device = engine.renderer.device.clone();

    let mesh_component = MeshComponent::new(
        vec![
            Vertex {
                position: [-1.0, -1.0, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 1.0, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
        ],
        vec![0, 1, 2],
    );

    let test_material = Material::new(
        "examples/shaders/vert.wgsl",
        "examples/shaders/frag.wgsl",
        Vec::new(),
        true,
        device,
    );

    scene.lock().unwrap().create_entity(
        0,
        true,
        vec![Box::new(mesh_component)],
        Some((vec![test_material], 0)),
    );

    engine.main_loop();
}
