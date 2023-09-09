use std::cell::{Ref, RefCell};

use nalgebra as na;
use sdl2::{keyboard::Scancode, mouse::RelativeMouseState};
use wgpu::util::DeviceExt;

#[rustfmt::skip]
const TRANSFORM_VECTOR: na::Vector3<f32> = na::Vector3::new(
    -1.0,
    -1.0,
    1.0
);

pub struct CameraManager {
    pub camera: RefCell<Camera>,
    pub camera_uniform: RefCell<CameraUniform>,
    pub buffer: Option<wgpu::Buffer>,
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl CameraManager {
    pub fn new(
        camera_position: na::Vector3<f32>,
        sensitivity: f32,
        movement_speed: f32,
        pitch: f32,
        yaw: f32,
        fovy: f32,
        near_plane: f32,
        far_plane: f32,
        window_w: f32,
        window_h: f32,
    ) -> Self {
        let camera = RefCell::new(Camera::new(
            camera_position.component_mul(&TRANSFORM_VECTOR),
            sensitivity,
            movement_speed,
            pitch,
            yaw,
            fovy,
            near_plane,
            far_plane,
            window_w,
            window_h,
        ));
        let camera_uniform = RefCell::new(CameraUniform::new(camera_position));

        CameraManager {
            camera,
            camera_uniform,
            buffer: None,
            bind_group_layout: None,
            bind_group: None,
        }
    }

    pub fn build_camera_resources(&mut self, device: &wgpu::Device) {
        let (buffer, bind_group_layout, bind_group) = self
            .camera_uniform
            .borrow_mut()
            .create_descriptor_and_buffer(device);
        self.buffer = Some(buffer);
        self.bind_group_layout = Some(bind_group_layout);
        self.bind_group = Some(bind_group);
    }
}

pub struct Camera {
    pub position: na::Vector3<f32>,
    pub affine_matrix: na::Matrix4<f32>,
    pub rotation_matrix: na::Matrix4<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub distance: f32,
    pub sensitivity: f32,
}

impl Camera {
    pub fn new(
        position: na::Vector3<f32>,
        sensitivity: f32,
        movement_speed: f32,
        pitch: f32,
        yaw: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
        window_w: f32,
        window_h: f32,
    ) -> Self {
        Camera {
            position,
            affine_matrix: na::Matrix4::identity(),
            rotation_matrix: na::Matrix4::identity(),
            pitch,
            yaw,
            aspect: window_w / window_h,
            fovy,
            znear,
            zfar,
            distance: movement_speed,
            sensitivity,
        }
    }

    fn build_view_projection_matrix(&self) -> na::Matrix4<f32> {
        let perspective = na::Perspective3::new(self.aspect, self.fovy, self.znear, self.zfar);
        let perspective_matrix = perspective.as_matrix();

        return perspective_matrix * self.affine_matrix;
    }

    pub fn update_affine_matrix(&mut self) {
        let transform_matrix = na::Matrix4::from(na::Translation3::from(self.position));

        let affine_matrix = self.rotation_matrix * transform_matrix;
        self.affine_matrix = affine_matrix;
    }

    fn update_rotation_matrix(&mut self) {
        let yaw_rotation_axis = na::Vector3::new(0.0, 1.0, 0.0);
        let yaw_matrix = na::Matrix4::new_rotation((yaw_rotation_axis * self.yaw).xyz());

        let pitch_rotation_axis = na::Vector3::new(1.0, 0.0, 0.0);

        let pitch_matrix = na::Matrix4::new_rotation((pitch_rotation_axis * self.pitch).xyz());

        self.rotation_matrix = pitch_matrix * yaw_matrix;
    }

    pub fn transform_camera(
        &mut self,
        scancodes: &Vec<Scancode>,
        mouse_state: &RelativeMouseState,
        relative_mouse: bool,
        delta_time: f32,
    ) {
        let distance = self.distance * delta_time;
        let sensitivity = self.sensitivity * delta_time;
        if scancodes.contains(&Scancode::W) {
            self.move_forward(distance);
        }
        if scancodes.contains(&Scancode::S) {
            self.move_backward(distance);
        }
        if scancodes.contains(&Scancode::D) {
            self.move_right(distance);
        }
        if scancodes.contains(&Scancode::A) {
            self.move_left(distance);
        }
        if scancodes.contains(&Scancode::Space) {
            self.move_up(distance);
        }
        if scancodes.contains(&Scancode::LCtrl) {
            self.move_down(distance);
        }

        if relative_mouse {
            self.rotate_yaw(mouse_state.x() as f32, sensitivity);
            self.rotate_pitch(mouse_state.y() as f32, sensitivity);
            self.update_rotation_matrix();
        }

        self.update_affine_matrix();
    }

    fn move_forward(&mut self, distance: f32) {
        self.position += (distance
            * self.rotation_matrix.try_inverse().unwrap()
            * na::Vector3::new(0.0, 0.0, 1.0).to_homogeneous())
        .xyz();
    }

    fn move_backward(&mut self, distance: f32) {
        self.move_forward(-distance);
    }

    fn move_right(&mut self, distance: f32) {
        self.position += (-distance
            * self.rotation_matrix.try_inverse().unwrap()
            * na::Vector3::new(1.0, 0.0, 0.0).to_homogeneous())
        .xyz();
    }

    fn move_left(&mut self, distance: f32) {
        self.move_right(-distance);
    }

    fn move_up(&mut self, distance: f32) {
        self.position.y -= distance;
    }

    fn move_down(&mut self, distance: f32) {
        self.move_up(-distance);
    }

    fn rotate_pitch(&mut self, rotation: f32, sensitivity: f32) {
        self.pitch += rotation * sensitivity;
    }

    fn rotate_yaw(&mut self, rotation: f32, sensitivity: f32) {
        self.yaw += rotation * sensitivity;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_pos: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
}
impl CameraUniform {
    pub fn new(position: na::Vector3<f32>) -> Self {
        CameraUniform {
            view_pos: position.to_homogeneous().into(),
            view_proj: na::Matrix4::identity().into(),
        }
    }
    pub fn update_view_proj(&mut self, camera: Ref<Camera>) {
        self.view_pos = camera.position.to_homogeneous().into();
        self.view_proj = camera.build_view_projection_matrix().into();
    }

    pub fn create_descriptor_and_buffer(
        self,
        device: &wgpu::Device,
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytemuck::cast_slice(&[self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                }],
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        (camera_buffer, camera_bind_group_layout, camera_bind_group)
    }
}
