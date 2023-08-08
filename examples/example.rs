use std::rc::Rc;

use nalgebra as na;

use gamezap::{
    camera::Camera,
    model::{Material, Mesh, ModelVertex},
    pipeline::MaterialMeshGroup,
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

    let camera = Camera::new(&renderer.device);

    renderer.set_camera(&camera);

    let material_layout =
        renderer
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("test_material_bind_group_layout"),
                entries: &[],
            });
    let material = Material::new(&renderer.device, "Test", None, None, material_layout);

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

    let meshes = vec![Mesh {
        name: "Test model".to_string(),
        vertex_buffer: model_vert_buffer,
        index_buffer: model_index_buffer,
        num_indices: model_indices.len() as u32,
        material: 0,
    }];

    let vertex_shader = wgpu::include_wgsl!("shaders/vert.wgsl");
    let fragment_shader = wgpu::include_wgsl!("shaders/frag.wgsl");

    let material_mesh_group =
        MaterialMeshGroup::new(material, meshes, renderer, vertex_shader, fragment_shader);
    renderer.pipeline_manager.material_mesh_groups.push(material_mesh_group);
    // renderer.material_mesh_groups.push(material_mesh_group);
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
        renderer.render().unwrap();
        ::std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn input(camera: &mut Camera, scancodes: &Vec<Scancode>, mouse_state: &RelativeMouseState) {
    camera.transform_camera(scancodes, mouse_state, true);
}
