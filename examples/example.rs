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
    GameZap,
};

use nalgebra as na;
use sdl2::keyboard::Scancode;

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
        // .hide_cursor()
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
        10.0,
        0.0,
        na::Vector3::new(1.0, 1.0, 1.0),
    );

    let camera_keyboard_controller = KeyboardInputComponent::new();

    let camera = scene_lock.create_entity(
        0,
        true,
        vec![
            Box::new(camera_component),
            Box::new(camera_transform),
            Box::new(camera_keyboard_controller),
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
    /* fn update(
        &mut self,
        _device: Arc<wgpu::Device>,
        _queue: Arc<wgpu::Queue>,
        component_map: AllComponents,
        engine_details: Arc<Mutex<gamezap::EngineDetails>>,
    ) {
        let components_arc = component_map.clone();
        let mut components = components_arc.lock().unwrap();
        // let transform_component = Scene::get_component_mut::<TransformComponent>(
        //     components.get_mut(&self.parent).unwrap(),
        // );
        /* if let Some(comp) = transform_component {
            let details_arc = engine_details.clone();
            let details = details_arc.lock().unwrap();
            for scancode in &details.pressed_scancodes {
                match scancode {
                    Scancode::W => comp.position.z += 0.5,
                    _ => {}
                }
            }
        } */
    } */

    fn update(
        &mut self,
        _device: Arc<wgpu::Device>,
        _queue: Arc<wgpu::Queue>,
        _component_map: AllComponents,
        engine_details: Arc<Mutex<gamezap::EngineDetails>>,
        concept_manager: Arc<Mutex<gamezap::ecs::concepts::ConceptManager>>,
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
