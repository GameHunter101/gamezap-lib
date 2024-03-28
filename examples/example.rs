use std::{
    any::TypeId,
    sync::{Arc, Mutex},
};

use gamezap::{
    ecs::{
        component::{ComponentId, ComponentSystem},
        components::{
            camera_component::CameraComponent, mesh_component::MeshComponent,
            transform_component::TransformComponent,
        },
        entity::EntityId,
        material::Material,
        scene::{AllComponents, Scene},
    },
    model::Vertex,
    texture::Texture,
    EngineSystems, GameZap, EngineDetails,
};

use nalgebra as na;
use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
};

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

    let scene = Arc::new(Mutex::new(Scene::default()));

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
        .hide_cursor()
        .scenes(scenes, 0)
        .build();

    let mut scene_lock = scene.lock().unwrap();
    let concept_manager = scene_lock.get_concept_manager();

    let device = engine.renderer.device.clone();
    let queue = engine.renderer.queue.clone();

    let mesh_component = MeshComponent::new(
        concept_manager.clone(),
        vec![
            Vertex {
                position: [-1.0, -1.0, 0.0],
                tex_coords: [0.0, 1.0],
                normal: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 1.0, 0.0],
                tex_coords: [0.5, 0.0],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
        ],
        vec![0, 1, 2],
    );

    let mesh_transform = TransformComponent::new(
        concept_manager.clone(),
        na::Vector3::new(0.1, 0.0, 1.0),
        0.0,
        0.0,
        0.0,
        na::Vector3::new(1.0, 1.0, 1.0),
    );

    let test_material = Material::new(
        "examples/shaders/vert.wgsl",
        "examples/shaders/frag.wgsl",
        vec![pollster::block_on(Texture::load_texture(
            "../textures/dude.png",
            &device.clone(),
            &queue,
            false,
        ))
        .unwrap()],
        true,
        device,
    );

    scene_lock.create_entity(
        0,
        true,
        vec![Box::new(mesh_component), Box::new(mesh_transform)],
        Some((vec![test_material], 0)),
    );

    let camera_component =
        CameraComponent::new_3d(concept_manager.clone(), (800, 600), 60.0, 0.1, 200.0);
    // let camera_component = CameraComponent::new_2d((800, 600));
    let camera_transform = TransformComponent::new(
        concept_manager,
        na::Vector3::new(0.1, 0.0, -1.0),
        0.0,
        0.0,
        0.0,
        na::Vector3::new(1.0, 1.0, 1.0),
    );

    let camera_keyboard_controller = KeyboardInputComponent::new();
    let camera_mouse_controller = MouseInputComponent::new();

    let camera = scene_lock.create_entity(
        0,
        true,
        vec![
            Box::new(camera_component),
            Box::new(camera_transform),
            Box::new(camera_keyboard_controller),
            Box::new(camera_mouse_controller),
        ],
        None,
    );

    scene_lock.set_active_camera(camera);
    drop(scene_lock);

    engine.main_loop();
}

#[derive(Debug, Clone)]
struct KeyboardInputComponent {
    parent: EntityId,
    id: ComponentId,
}
impl KeyboardInputComponent {
    fn new() -> Self {
        KeyboardInputComponent {
            parent: EntityId::MAX,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
        }
    }
}

impl ComponentSystem for KeyboardInputComponent {
    fn update(
        &mut self,
        _device: Arc<wgpu::Device>,
        _queue: Arc<wgpu::Queue>,
        _component_map: AllComponents,
        engine_details: Arc<Mutex<gamezap::EngineDetails>>,
        _engine_systems: Arc<Mutex<EngineSystems>>,
        concept_manager: Arc<Mutex<gamezap::ecs::concepts::ConceptManager>>,
        _active_camera_id: Option<EntityId>,
    ) {
        let mut concept_manager = concept_manager.lock().unwrap();
        let position_concept = concept_manager
            .get_concept_mut::<na::Vector3<f32>>(
                (self.parent, TypeId::of::<TransformComponent>(), 0),
                "position".to_string(),
            )
            .unwrap();

        let details = engine_details.lock().unwrap();
        let speed = 0.1;

        for scancode in &details.pressed_scancodes {
            match scancode {
                Scancode::W => {
                    position_concept.z += speed;
                }
                Scancode::S => {
                    position_concept.z -= speed;
                }
                Scancode::A => {
                    position_concept.x += speed;
                }
                Scancode::D => {
                    position_concept.x -= speed;
                }
                Scancode::LCtrl => {
                    position_concept.y += speed;
                }
                Scancode::Space => {
                    position_concept.y -= speed;
                }
                _ => {}
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn update_metadata(&mut self, parent: EntityId, same_component_count: u32) {
        self.parent = parent;
        self.id.0 = parent;
        self.id.2 = same_component_count;
    }

    fn get_parent_entity(&self) -> EntityId {
        self.parent
    }

    fn get_id(&self) -> ComponentId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct MouseInputComponent {
    parent: EntityId,
    id: ComponentId,
}

impl MouseInputComponent {
    fn new() -> Self {
        MouseInputComponent {
            parent: EntityId::MAX,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
        }
    }
}

impl ComponentSystem for MouseInputComponent {
    fn update(
        &mut self,
        _device: Arc<wgpu::Device>,
        _queue: Arc<wgpu::Queue>,
        _component_map: AllComponents,
        engine_details: Arc<Mutex<gamezap::EngineDetails>>,
        engine_systems: Arc<Mutex<EngineSystems>>,
        concept_manager: Arc<Mutex<gamezap::ecs::concepts::ConceptManager>>,
        active_camera_id: Option<EntityId>,
    ) {
        let mut concept_manager = concept_manager.lock().unwrap();
        let pitch = *concept_manager
            .get_concept::<f32>(
                (
                    active_camera_id.unwrap(),
                    TypeId::of::<TransformComponent>(),
                    0,
                ),
                "pitch".to_string(),
            )
            .unwrap();

        let yaw = *concept_manager
            .get_concept::<f32>(
                (
                    active_camera_id.unwrap(),
                    TypeId::of::<TransformComponent>(),
                    0,
                ),
                "yaw".to_string(),
            )
            .unwrap();

        let systems = engine_systems.lock().unwrap();
        let sdl_context = systems.sdl_context.lock().unwrap();
        let mouse = sdl_context.mouse();
        let is_hidden = mouse.relative_mouse_mode();
        let details = engine_details.lock().unwrap();
        if is_hidden {
            if let Some(mouse_state) = details.mouse_state.0 {
                concept_manager
                    .modify_concept(
                        (
                            active_camera_id.unwrap(),
                            TypeId::of::<TransformComponent>(),
                            0,
                        ),
                        "pitch".to_string(),
                        pitch + mouse_state.x() as f32 / 100.0,
                    )
                    .unwrap();

                concept_manager
                    .modify_concept(
                        (
                            active_camera_id.unwrap(),
                            TypeId::of::<TransformComponent>(),
                            0,
                        ),
                        "yaw".to_string(),
                        yaw + mouse_state.y() as f32 / 100.0,
                    )
                    .unwrap();
            }
        }
    }

    fn on_event(
        &self,
        event: &Event,
        _component_map: &std::collections::HashMap<
            EntityId,
            Vec<gamezap::ecs::component::Component>,
        >,
        _concept_manager: &gamezap::ecs::concepts::ConceptManager,
        _active_camera_id: Option<EntityId>,
        _engine_details: &EngineDetails,
        engine_systems: &EngineSystems,
    ) {
        if let Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } = event
        {
            let context = engine_systems.sdl_context.lock().unwrap();
            let is_cursor_showing = context.mouse().is_cursor_showing();
            context.mouse().show_cursor(!is_cursor_showing);
            context.mouse().set_relative_mouse_mode(is_cursor_showing);
        }
    }

    fn get_parent_entity(&self) -> EntityId {
        self.parent
    }
    fn get_id(&self) -> ComponentId {
        self.id
    }

    fn update_metadata(&mut self, parent: EntityId, same_component_count: u32) {
        self.parent = parent;
        self.id.0 = parent;
        self.id.2 = same_component_count;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
