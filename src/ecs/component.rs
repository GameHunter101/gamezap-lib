use std::sync::{Arc, Mutex};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Device, Queue, Surface,
};

use crate::{model::Vertex, texture::Texture};

pub enum Component {
    Normal(Box<dyn ComponentSystem>),
    Material(MaterialComponent),
}

impl Component {
    pub fn this_entity(&self) -> &Vec<usize> {
        match self {
            Self::Normal(comp) => comp.this_entity(),
            Self::Material(mat) => mat.this_entity(),
        }
    }
}

pub trait ComponentSystem {
    fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        entity_components: &mut Vec<Component>,
    );
    fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        entity_components: &mut Vec<Component>,
        smaa_target: Arc<Mutex<smaa::SmaaTarget>>,
        surface: Arc<Surface>,
        depth_texture: Arc<Texture>,
    );
    fn this_entity(&self) -> &Vec<usize>;
}

pub struct MeshComponent {
    entity: Vec<usize>,
    vertices: Vec<Vertex>,
    indices: Vec<u64>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
}

impl ComponentSystem for MeshComponent {
    fn initialize(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        entity_components: &mut Vec<Component>,
    ) {
        self.vertex_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("Entity {:?} Vertex Buffer", self.entity)),
            contents: &bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));

        self.index_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("Entity {:?} Vertex Buffer", self.entity)),
            contents: &bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        }));
    }

    fn update(
        &mut self,
        device: Arc<Device>,
        queue: Arc<Queue>,
        entity_components: &mut Vec<Component>,
        smaa_target: Arc<Mutex<smaa::SmaaTarget>>,
        surface: Arc<Surface>,
        depth_texture: Arc<crate::texture::Texture>,
    ) {
        let output = surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut smaa_binding = smaa_target.lock().unwrap();
        let smaa_frame = smaa_binding.start_frame(&device, &queue, &view);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("Entity {:?} Command Encoder", self.entity)),
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("Entity {:?} Render Pass", self.entity)),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &smaa_frame,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
    }

    fn this_entity(&self) -> &Vec<usize> {
        &self.entity
    }
}

pub struct MaterialComponent {
    entity: Vec<usize>,
    vertex_shader_path: String,
    fragment_shader_path: String,
    textures: Vec<Texture>,
    enabled: bool,
    id: (String, String, usize),
}

impl MaterialComponent {
    fn new(
        entity: Vec<usize>,
        vertex_shader_path: &str,
        fragment_shader_path: &str,
        textures: Vec<Texture>,
        enabled: bool,
    ) -> Self {
        let id = (
                vertex_shader_path.to_string(),
                fragment_shader_path.to_string(),
                textures.len(),
            );
        Self {
            entity,
            vertex_shader_path: vertex_shader_path.to_string(),
            fragment_shader_path: fragment_shader_path.to_string(),
            textures,
            enabled,
            id,
        }
    }

    fn this_entity(&self) -> &Vec<usize> {
        &self.entity
    }
}
