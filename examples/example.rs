use algoe::{bivector::Bivector, rotor::Rotor3};
use components::{
    compute_monitor_component::ComputeMonitorComponent,
    keyboard_input_component::KeyboardInputComponent, mouse_input_component::MouseInputComponent,
    transparency_component::TransparencyComponent, ui_component::UiComponent,
};
use gamezap::{
    ecs::{
        components::{
            camera_component::CameraComponent, mesh_component::MeshComponent,
            physics_component::PhysicsComponent, transform_component::TransformComponent,
        },
        material::Material,
        scene::Scene,
    },
    model::Vertex,
    pipeline::{ComputeData, ComputePipelineType, ComputeTextureData},
    texture::Texture,
    GameZap,
};

use nalgebra as na;

extern crate gamezap;

pub mod components {
    pub mod compute_monitor_component;
    pub mod keyboard_input_component;
    pub mod mouse_input_component;
    pub mod transparency_component;
    pub mod ui_component;
}

#[tokio::main]
async fn main() {
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

    let mut engine = GameZap::builder()
        .window_and_renderer(
            sdl_context,
            video_subsystem,
            event_pump,
            window,
            wgpu::Color {
                r: 0.7,
                g: 0.2,
                b: 0.2,
                a: 1.0,
            },
        )
        .antialiasing()
        .hide_cursor()
        .build()
        .await;

    let mut scene = Scene::default();
    let concept_manager = scene.get_concept_manager();

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
                position: [-1.0, 1.0, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
        ],
        vec![0, 1, 2, 1, 2, 3],
    );

    let mesh_transform = TransformComponent::new(
        concept_manager.clone(),
        na::Vector3::new(0.1, 0.0, 1.0),
        Rotor3::default(),
        na::Vector3::new(1.0, 1.0, 1.0),
    );

    let test_material = Material::new(
        "examples/shaders/vert.wgsl",
        "examples/shaders/frag.wgsl",
        vec![Texture::load_texture(
            "assets/testing_textures/texture.png",
            false,
            &device.clone(),
            &queue,
            false,
        )
        .await
        .unwrap()],
        None,
        true,
        device.clone(),
    );

    scene.create_entity(
        0,
        true,
        vec![Box::new(mesh_transform), Box::new(mesh_component)],
        Some((vec![test_material], 0)),
    );

    let sword_mesh =
        MeshComponent::from_obj(concept_manager.clone(), "assets\\models\\blade.obj", false)
            .unwrap();

    let sword_transform = TransformComponent::new(
        concept_manager.clone(),
        na::Vector3::new(1.0, 1.0, 0.0),
        (Bivector::new(0.0, 1.0, 0.0) * -std::f32::consts::FRAC_PI_2).exponentiate(),
        na::Vector3::new(1.0, 1.0, 1.0),
    );

    let dude_texture = Texture::load_texture(
        "assets/testing_textures/dude.png",
        false,
        &device.clone(),
        &queue,
        false,
    )
    .await
    .unwrap();

    let sword_material = Material::new(
        "examples/shaders/vert.wgsl",
        "examples/shaders/frag.wgsl",
        vec![dude_texture],
        None,
        true,
        device.clone(),
    );

    /* dbg!(
        angular_velocity.normalized(),
        (angular_velocity * 100.0).normalized()
    ); */

    let sword_physics = PhysicsComponent::new(
        concept_manager.clone(),
        na::Vector3::new(0.0, 0.0, 0.0),
        na::Vector3::new(0.0, 0.0, 0.0),
        1.0,
        // angular_velocity / 100.0,
        // 0.001 * ultraviolet::Bivec3::unit_xy(),
        Bivector::default(),
        Bivector::default(),
        // Bivec3::zero(),
        // Bivec3::default(),
        //
        // ultraviolet::Bivec3::from_angle_plane(0.01, ultraviolet::Bivec3::unit_yz()),
        // ultraviolet::Rotor3::default(),
    );

    scene.create_entity(
        0,
        true,
        vec![
            Box::new(sword_mesh),
            Box::new(sword_transform),
            Box::new(sword_physics),
        ],
        Some((vec![sword_material], 0)),
    );

    let cube_material = Material::new(
        "examples/shaders/vert.wgsl",
        "examples/shaders/frag2.wgsl",
        vec![Texture::load_texture(
            "assets/testing_textures/dude.png",
            false,
            &device.clone(),
            &queue,
            false,
        )
        .await
        .unwrap()],
        Some(bytemuck::cast_slice(&[0.0_f32])),
        true,
        device.clone(),
    );

    let cube_mesh = MeshComponent::from_obj(
        concept_manager.clone(),
        "assets\\models\\basic_cube.obj",
        false,
    )
    .unwrap();

    let cube_transform = TransformComponent::new(
        concept_manager.clone(),
        na::Vector3::new(0.0, 0.0, 0.0),
        Rotor3::default(),
        /* 0.0,
        0.0,
        0.0, */
        na::Vector3::new(0.1, 0.1, 0.1),
    );

    let cube_transparency = TransparencyComponent::default();

    scene.create_entity(
        0,
        true,
        vec![
            Box::new(cube_mesh),
            Box::new(cube_transform),
            Box::new(cube_transparency),
        ],
        Some((vec![cube_material], 0)),
    );

    let camera_component =
        CameraComponent::new_3d(concept_manager.clone(), (800, 600), 60.0, 0.01, 200.0);
    let camera_transform = TransformComponent::new(
        concept_manager.clone(),
        na::Vector3::new(0.1, 0.0, -1.0),
        // Bivector::new(0.0, 0.0, -1.0).exponentiate(),
        Rotor3::default(),
        na::Vector3::new(1.0, 1.0, 1.0),
    );

    let camera_keyboard_controller = KeyboardInputComponent::default();
    let camera_mouse_controller = MouseInputComponent::default();

    let camera = scene.create_entity(
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

    let test_compute_pipeline_index = scene.create_compute_pipeline(
        device.clone(),
        queue.clone(),
        "examples/shaders/compute_texture.wgsl",
        (128, 1, 1),
        ComputePipelineType::<[u32; 128]> {
            input_data: ComputeData::TextureData(ComputeTextureData::Dimensions((
                1000, 1000,
            ))),
            output_data_type: gamezap::pipeline::ComputeOutput::Array(std::mem::size_of::<[u32;128]>() as u64)
        },
        /* "examples/shaders/compute_2.wgsl",
        (6,1,1),
        ComputeData::ArrayData([5.0, 6.0, 10.0, 4.0, 0.1, 12.0_f32]), */
    );

    let compute_monitor_component =
        ComputeMonitorComponent::new(test_compute_pipeline_index.unwrap());

    let _compute_entity =
        scene.create_entity(0, true, vec![Box::new(compute_monitor_component)], None);

    scene.set_active_camera(camera);

    let ui_component = UiComponent::new("assets/fonts/inter.ttf");

    let _ui_entity = scene.create_entity(0, true, vec![Box::new(ui_component)], None);

    engine.create_scene(scene);

    engine.main_loop();
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
struct TestUniform {
    coefficient: f32,
}
