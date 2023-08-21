use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use nalgebra as na;

use gamezap::{
    camera::{Camera, CameraUniform},
    materials::{Material},
    model::{Mesh, ModelVertex},
    pipeline_manager::PipelineManager,
    GameZap,
};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
    mouse::RelativeMouseState,
};
use wgpu::util::DeviceExt;

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

    let renderer = &mut engine.renderer;

    let pipeline_manager = Arc::new(Mutex::new(PipelineManager::init()));
    let camera_position = na::Vector3::new(0.0,0.0,0.0);
    let camera_uniform = CameraUniform::new(camera_position);
    let camera = Arc::new(Mutex::new(Camera::new(camera_position,camera_uniform,&renderer.device)));

    renderer.set_camera(camera.clone(),camera_uniform);
    renderer.set_pipeline_manager(pipeline_manager.clone());

    let mut material = Material::new(&renderer.device, "Test", None, None);

    let model_vertices = vec![
        ModelVertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        ModelVertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        ModelVertex {
            position: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
    ];

    let model_vert_buffer = renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Test model vertex buffer"),
            contents: &bytemuck::cast_slice(&model_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

    let model_indices: [u16; 3] = [0, 1, 2];

    let model_index_buffer =
        renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Test model index buffer"),
                contents: &bytemuck::cast_slice(&model_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

    let mesh = Mesh {
        name: "Test model".to_string(),
        vertex_buffer: model_vert_buffer,
        index_buffer: model_index_buffer,
        num_indices: model_indices.len() as u32,
        material: 0,
    };
    material.meshes.push(mesh);
    let pipeline_manager_clone = pipeline_manager.clone();
    pipeline_manager_clone
        .lock()
        .unwrap()
        .materials
        .no_texture_materials
        .push(material);

    let vertex_shader = wgpu::include_wgsl!("shaders/vert.wgsl");
    let fragment_shader = wgpu::include_wgsl!("shaders/frag.wgsl");

    renderer.create_pipelines();

    'running: loop {
        for event in engine.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => renderer.resize((width as u32, height as u32)),
                _ => {}
            }
        }
        let scancodes = engine
            .event_pump
            .keyboard_state()
            .pressed_scancodes()
            .collect::<Vec<_>>();
        let mouse_state = engine.event_pump.relative_mouse_state();
        input(camera.clone(), &scancodes, &mouse_state);
        renderer.update_buffers();
        renderer.render().unwrap();
        ::std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn input(camera: Arc<Mutex<Camera>>, scancodes: &Vec<Scancode>, mouse_state: &RelativeMouseState) {
    let mut camera = camera.lock().unwrap();
    camera.transform_camera(scancodes, mouse_state, true);
}