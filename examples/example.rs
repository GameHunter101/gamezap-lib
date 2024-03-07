use std::sync::Arc;

use gamezap::{
    ecs::{
        component::{CameraComponent, ComponentSystem},
        entity::EntityId,
        scene::Scene,
    },
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

    let mut scene = Scene::new();

    let test_component = TestComponent { entity_id: 0 };

    scene.create_entity(0, true, vec![Box::new(test_component)], None);

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

#[derive(Debug)]
struct TestComponent {
    entity_id: EntityId,
}

impl ComponentSystem for TestComponent {
    fn this_entity(&self) -> &EntityId {
        &self.entity_id
    }
    fn update(
        &mut self,
        _device: Arc<wgpu::Device>,
        _queue: Arc<wgpu::Queue>,
        _all_components: Arc<
            std::sync::Mutex<
                std::collections::HashMap<EntityId, Vec<gamezap::ecs::component::Component>>,
            >,
        >,
        _engine_details: Arc<std::sync::Mutex<gamezap::EngineDetails>>,
    ) {
        println!("Hello");
    }
}
