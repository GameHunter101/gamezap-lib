use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use nalgebra as na;

use gamezap::{
    model::{Mesh, MeshTransform, Vertex},
    module_manager::ModuleManager,
    renderer::Renderer,
    texture::Texture,
    EngineDetails, GameZap,
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
        .camera_manager(
            camera_position,
            0.8,
            2.0,
            0.0,
            0.0,
            45.0,
            0.1,
            100.0,
            window_size.0 as f32,
            window_size.1 as f32,
        )
        .mesh_manager()
        .build();

    let engine = RefCell::new(
        GameZap::builder()
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
            .build(),
    );

    let mut engine_borrow = engine.borrow_mut();
    let renderer = engine_borrow.renderer.borrow();

    let mut material_manager = renderer.module_manager.material_manager.borrow_mut();

    let renderer_device = &renderer.device;
    let renderer_queue = &renderer.queue;

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
        let mut mesh_manager = renderer
            .module_manager
            .mesh_manager
            .as_ref()
            .unwrap()
            .borrow_mut();

        mesh_manager.plain_pipeline_models.push(mesh);
        mesh_manager.diffuse_pipeline_models.push(second_mesh);
        mesh_manager.diffuse_pipeline_models.push(third_mesh);
    }

    renderer.prep_renderer();

    drop(renderer_queue);
    drop(renderer_device);
    drop(renderer);
    engine_borrow.main_loop(vec![Box::new(input)]);
}

fn input(engine_details: RefMut<EngineDetails>, renderer: RefMut<Renderer>) {
    let camera_manager = &renderer.module_manager.camera_manager;
    if let Some(camera_manager) = camera_manager {
        let camera_manager = camera_manager.borrow();
        let mut camera = camera_manager.camera.borrow_mut();
        if let Some(mouse_state) = engine_details.mouse_state.0 {
            camera.transform_camera(
                &engine_details.pressed_scancodes,
                &mouse_state,
                true,
                engine_details.last_frame_duration.as_seconds_f32(),
            );
        }
    }
}
