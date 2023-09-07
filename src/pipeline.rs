use std::cell::Ref;

use crate::{
    camera::CameraManager,
    materials::MaterialManager,
    model::{Mesh, Vertex, VertexData},
    texture::Texture,
};

pub struct PipelineManager {
    pub plain_pipeline: Option<Pipeline>,
    pub diffuse_texture_pipeline: Option<Pipeline>,
    pub diffuse_normal_texture_pipeline: Option<Pipeline>,
}

impl PipelineManager {
    pub fn init() -> Self {
        PipelineManager {
            plain_pipeline: None,
            diffuse_texture_pipeline: None,
            diffuse_normal_texture_pipeline: None,
        }
    }

    pub fn create_pipelines(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        material_manager: Ref<MaterialManager>,
        camera_manager: Option<Ref<CameraManager>>,
    ) {
        if material_manager.plain_materials.len() > 0 {
            if self.plain_pipeline.is_none() {
                let mut layouts = vec![&material_manager.plain_materials[0].bind_group_layout];
                if let Some(camera_manager) = &camera_manager {
                    layouts.push(&camera_manager.bind_group_layout.as_ref().unwrap());
                }
                let pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("NoTexturePipelineLayout"),
                        bind_group_layouts: &layouts,
                        push_constant_ranges: &[],
                    });

                let vertex_shader = wgpu::include_wgsl!("../examples/shaders/vert.wgsl");
                let fragment_shader = wgpu::include_wgsl!("../examples/shaders/frag.wgsl");

                self.plain_pipeline = Some(Pipeline::new(
                    "Plain pipeline",
                    device,
                    &pipeline_layout,
                    format,
                    Some(Texture::DEPTH_FORMAT),
                    &[Vertex::desc(), Mesh::desc()],
                    vertex_shader,
                    fragment_shader,
                ));
            }
        }

        if material_manager.diffuse_texture_materials.len() > 0 {
            if self.diffuse_texture_pipeline.is_none() {
                let mut layouts =
                    vec![&material_manager.diffuse_texture_materials[0].bind_group_layout];
                if let Some(camera_manager) = &camera_manager {
                    layouts.push(&camera_manager.bind_group_layout.as_ref().unwrap());
                }
                let pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("DiffuseTexturePipelineLayout"),
                        bind_group_layouts: &layouts,
                        push_constant_ranges: &[],
                    });

                let vertex_shader = wgpu::include_wgsl!("./default-shaders/texture_vert.wgsl");
                let fragment_shader = wgpu::include_wgsl!("./default-shaders/texture_frag.wgsl");

                self.diffuse_texture_pipeline = Some(Pipeline::new(
                    "Diffuse pipeline",
                    device,
                    &pipeline_layout,
                    format,
                    Some(Texture::DEPTH_FORMAT),
                    &[Vertex::desc(), Mesh::desc()],
                    vertex_shader,
                    fragment_shader,
                ))
            }
        }
    }
}

#[derive(Debug)]
pub enum PipelineType {
    Plain,
    DiffuseTexture,
    NormalDiffuseTexture,
}

#[derive(Debug)]
pub struct Pipeline {
    pub pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
        name: &str,
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        color_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        vertex_shader: wgpu::ShaderModuleDescriptor,
        fragment_shader: wgpu::ShaderModuleDescriptor,
    ) -> Self {
        let vertex_shader = device.create_shader_module(vertex_shader);
        let fragment_shader = device.create_shader_module(fragment_shader);

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(name),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "main",
                buffers: vertex_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Pipeline {
            pipeline: render_pipeline,
        }
    }
}
