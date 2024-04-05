use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use gamezap::{
    ecs::{
        component::{Component, ComponentId, ComponentSystem},
        components::{
            camera_component::CameraComponent, mesh_component::MeshComponent,
            transform_component::TransformComponent,
        },
        concepts::ConceptManager,
        entity::EntityId,
        material::Material,
        scene::{AllComponents, Scene},
    },
    model::Vertex,
    texture::Texture,
    EngineDetails, EngineSystems, GameZap,
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
    let window = video_subsystem
        .window(application_title, window_size.0, window_size.1)
        .resizable()
        .build()
        .unwrap();

    let scene = Arc::new(Mutex::new(Scene::default()));

    let scenes = vec![scene.clone()];

    let mut engine = GameZap::builder()
        .window_and_renderer(
            sdl_context,
            video_subsystem,
            event_pump,
            window,
            wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
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
            "../assets/testing_textures/dude.png",
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

    /* {
        let systems = engine.systems.lock().unwrap();
        let font_context = systems.font_context.clone();
        // let mut canvas = engine.canvas.lock().unwrap();
        let texture_creator = canvas.texture_creator();

        let mut font = font_context
            .load_font(
                std::env::current_dir()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    + "\\assets\\fonts\\inter.ttf",
                16,
            )
            .unwrap();

        font.set_style(sdl2::ttf::FontStyle::BOLD);
        let surface = font
            .render("Hello world!")
            .blended(Color::RGBA(255, 0, 0, 10))
            .unwrap();

        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();

        let mut other_surface =
            sdl2::surface::Surface::new(200, 200, sdl2::pixels::PixelFormatEnum::RGBA8888).unwrap();

        other_surface
            .fill_rect(
                sdl2::rect::Rect::new(150, 150, 50, 50),
                Color::RGBA(100, 100, 255, 40),
            )
            .unwrap();

        let other_texture = texture_creator
            .create_texture_from_surface(other_surface)
            .unwrap();

        canvas.set_draw_color(Color::RGBA(0, 255, 0, 100));
        canvas.clear();

        let sdl2::render::TextureQuery { width, height, .. } = texture.query();

        let sdl2::render::TextureQuery {
            width: width_2,
            height: height_2,
            ..
        } = other_texture.query();

        canvas
            .copy(
                &texture,
                None,
                Some(sdl2::rect::Rect::new(100, 100, width, height)),
            )
            .unwrap();

        canvas
            .copy(
                &other_texture,
                None,
                Some(sdl2::rect::Rect::new(100, 100, width_2, height_2)),
            )
            .unwrap();

        canvas.present();
    } */

    // std::thread::sleep(std::time::Duration::from_secs(10));
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
        component_map: AllComponents,
        engine_details: Arc<Mutex<EngineDetails>>,
        _engine_systems: Arc<Mutex<EngineSystems>>,
        concept_manager: Arc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
    ) {
        let mut concept_manager = concept_manager.lock().unwrap();

        let transform_component =
            Scene::get_component::<TransformComponent>(component_map.get(&self.parent).unwrap());
        let camera_rotation_matrix = match transform_component {
            Some(transform) => transform.create_rotation_matrix(&concept_manager),
            None => na::Matrix4::identity(),
        };

        let position_concept = concept_manager
            .get_concept_mut::<na::Vector3<f32>>(
                (self.parent, TypeId::of::<TransformComponent>(), 0),
                "position".to_string(),
            )
            .unwrap();

        let details = engine_details.lock().unwrap();
        let speed = 0.1;

        let forward_vector = (camera_rotation_matrix
            * na::Vector3::new(0.0, 0.0, 1.0).to_homogeneous())
        .xyz()
        .normalize();

        let left_vector = forward_vector.cross(&-na::Vector3::y_axis()).normalize();

        for scancode in &details.pressed_scancodes {
            match scancode {
                Scancode::W => {
                    *position_concept += forward_vector * speed;
                }
                Scancode::S => {
                    *position_concept -= forward_vector * speed;
                }
                Scancode::A => {
                    *position_concept -= left_vector * speed;
                }
                Scancode::D => {
                    *position_concept += left_vector * speed;
                }
                Scancode::LCtrl => {
                    position_concept.y -= speed;
                }
                Scancode::Space => {
                    position_concept.y += speed;
                }
                _ => {}
            }
        }
    }

    fn on_event(
        &self,
        event: &Event,
        _component_map: &HashMap<EntityId, Vec<Component>>,
        _concept_manager: &ConceptManager,
        _active_camera_id: Option<EntityId>,
        _engine_details: &EngineDetails,
        engine_systems: &EngineSystems,
    ) {
        let context = engine_systems.sdl_context.lock().unwrap();
        if let Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } = event
        {
            let is_cursor_visible = context.mouse().is_cursor_showing();
            context.mouse().set_relative_mouse_mode(is_cursor_visible);
            context.mouse().show_cursor(!is_cursor_visible);
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
        engine_details: Arc<Mutex<EngineDetails>>,
        engine_systems: Arc<Mutex<EngineSystems>>,
        concept_manager: Arc<Mutex<ConceptManager>>,
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

                if ((yaw - std::f32::consts::FRAC_PI_2).abs() <= 0.1 && mouse_state.y() > 0)
                    || ((yaw + std::f32::consts::FRAC_PI_2).abs() <= 0.1 && mouse_state.y() < 0)
                {
                    return;
                }
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
