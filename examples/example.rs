use std::{cell::RefCell, rc::Rc};

use nalgebra as na;

use gamezap::{
    camera::CameraManager,
    model::{Mesh, MeshTransform, Vertex},
    module_manager::ModuleManager,
    texture::Texture,
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

    let camera_position = na::Vector3::new(-2.0, 0.0, -5.0);
    let module_manager = ModuleManager::builder()
        .camera_manager(camera_position, 0.0, 0.0, 45.0, 0.005)
        .mesh_manager()
        .build();

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
        .module_manager(module_manager)
        .antialiasing()
        .build();

    let renderer = RefCell::new(engine.renderer);

    let renderer_borrow = renderer.borrow();
    let mut material_manager = renderer_borrow.module_manager.material_manager.borrow_mut();

    let renderer_device = &renderer_borrow.device;
    let renderer_queue = &renderer_borrow.queue;

    let first_material =
        material_manager.new_material("First material", renderer_device, None, None);
    let second_material = material_manager.new_material(
        "Second material",
        renderer_device,
        Some(
            pollster::block_on(Texture::load_texture(
                "texture.png",
                renderer_device,
                renderer_queue,
                false,
            ))
            .unwrap(),
        ),
        None,
    );

    let third_material = material_manager.new_material(
        "Second material",
        renderer_device,
        Some(
            pollster::block_on(Texture::load_texture(
                "atlas.png",
                renderer_device,
                renderer_queue,
                false,
            ))
            .unwrap(),
        ),
        None,
    );
    drop(material_manager);

    let first_model_vertices = vec![
        Vertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
    ];

    let first_model_indices: [u32; 3] = [0, 1, 2];

    let first_model_vert_buffer =
        renderer_device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Test model vertex buffer"),
            contents: &bytemuck::cast_slice(&first_model_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

    let first_model_index_buffer =
        renderer_device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Test model index buffer"),
            contents: &bytemuck::cast_slice(&first_model_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

    let mesh = Mesh::new(
        renderer_device,
        "First model".to_string(),
        first_model_vert_buffer,
        first_model_index_buffer,
        first_model_indices.len() as u32,
        MeshTransform::new(
            na::Vector3::new(1.0, 0.0, 0.0),
            na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
        ),
        first_material.1,
    );

    let second_model_vertices = vec![
        Vertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.778, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.778, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
    ];

    let second_model_indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

    let second_vert_buffer =
        renderer_device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Second model vertex buffer"),
            contents: &bytemuck::cast_slice(&second_model_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

    let second_index_buffer =
        renderer_device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Second model index buffer"),
            contents: &bytemuck::cast_slice(&second_model_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

    let second_mesh = Mesh::new(
        renderer_device,
        "Second model".to_string(),
        second_vert_buffer,
        second_index_buffer,
        second_model_indices.len() as u32,
        MeshTransform::new(
            na::Vector3::new(-1.0, 0.0, 0.0),
            na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
        ),
        second_material.1,
    );

    let third_model_vertices = vec![
        Vertex {
            position: [0.0, 1.041, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 1.041, 0.0],
            tex_coords: [1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
    ];

    let third_model_indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

    let third_vert_buffer = renderer_device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Second model vertex buffer"),
        contents: &bytemuck::cast_slice(&third_model_vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let third_index_buffer =
        renderer_device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Second model index buffer"),
            contents: &bytemuck::cast_slice(&third_model_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

    let third_mesh = Mesh::new(
        renderer_device,
        "Second model".to_string(),
        third_vert_buffer,
        third_index_buffer,
        third_model_indices.len() as u32,
        MeshTransform::new(
            na::Vector3::new(-3.0, 0.0, 0.0),
            na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
        ),
        third_material.1,
    );

    {
        let mut mesh_manager = renderer_borrow
            .module_manager
            .mesh_manager
            .as_ref()
            .unwrap()
            .borrow_mut();

        mesh_manager.plain_pipeline_models.push(mesh);
        mesh_manager.diffuse_pipeline_models.push(second_mesh);
        mesh_manager.diffuse_pipeline_models.push(third_mesh);
    }

    renderer.borrow().prep_renderer();

    drop(renderer_queue);
    drop(renderer_device);
    drop(renderer_borrow);
    'running: loop {
        let mut renderer_borrow = renderer.borrow_mut();
        for event in engine.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => renderer_borrow.resize((width as u32, height as u32)),
                _ => {}
            }
        }
        let scancodes = engine
            .event_pump
            .keyboard_state()
            .pressed_scancodes()
            .collect::<Vec<_>>();
        let mouse_state = engine.event_pump.relative_mouse_state();
        input(
            renderer_borrow.module_manager.camera_manager.as_ref(),
            &scancodes,
            &mouse_state,
        );
        renderer_borrow.update_buffers();
        renderer_borrow.render().unwrap();
        ::std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn input(
    camera_manager: Option<&RefCell<CameraManager>>,
    scancodes: &Vec<Scancode>,
    mouse_state: &RelativeMouseState,
) {
    if let Some(camera_manager) = camera_manager {
        let camera_manager = camera_manager.borrow();
        let mut camera = camera_manager.camera.borrow_mut();
        camera.transform_camera(scancodes, mouse_state, true);
    }
}
