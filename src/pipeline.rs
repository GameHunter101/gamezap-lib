#![allow(clippy::too_many_arguments)]
use std::{fmt::Debug, sync::Arc};
use wgpu::{Device, PipelineLayout, RenderPipeline, ShaderStages};

use crate::{
    ecs::{components::camera_component::CameraComponent, material::MaterialId},
    texture::Texture,
};

#[derive(Debug)]
pub enum PipelineError {
    PathNotFound(String),
}

#[derive(Debug)]
pub enum PipelineType {
    Plain,
    DiffuseTexture,
    NormalDiffuseTexture,
}

#[derive(Debug)]
pub struct Pipeline {
    pipeline: RenderPipeline,
    id: MaterialId,
}

impl Pipeline {
    pub fn new(
        device: Arc<Device>,
        color_format: wgpu::TextureFormat,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        id: &MaterialId,
    ) -> Self {
        let vertex_descriptor = Pipeline::load_shader_module_descriptor(&id.0).unwrap();
        let fragment_descriptor = Pipeline::load_shader_module_descriptor(&id.1).unwrap();
        let vertex_shader = device.create_shader_module(vertex_descriptor);
        let fragment_shader = device.create_shader_module(fragment_descriptor);

        let layout = Pipeline::create_pipeline_layout(id, device.clone());

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{id:?} Pipeline")),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "main",
                buffers: vertex_layouts,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
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
            cache: None,
        });

        Pipeline {
            pipeline: render_pipeline,
            id: id.clone(),
        }
    }

    pub fn create_pipeline_layout(material_id: &MaterialId, device: Arc<Device>) -> PipelineLayout {
        let texture_bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry> = if material_id.2
            == 0
        {
            Vec::new()
        } else {
            (0..material_id.2)
                .flat_map(|i| {
                    [
                        wgpu::BindGroupLayoutEntry {
                            binding: i as u32 * 2,
                            visibility: ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: i as u32 * 2 + 1,
                            visibility: ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ]
                })
                .collect::<Vec<wgpu::BindGroupLayoutEntry>>()
        };
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{material_id:?} Texture Bind Group Layout")),
                entries: &texture_bind_group_layout_entries,
            });

        let uniform_buffer_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{material_id:?} Uniform Buffer Bind Group Layout")),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let camera_bind_group_layout = CameraComponent::camera_bind_group_layout(device.clone());

        let all_layouts = if material_id.3 {
            vec![
                &texture_bind_group_layout,
                &camera_bind_group_layout,
                &uniform_buffer_bind_group_layout,
            ]
        } else {
            vec![&texture_bind_group_layout, &camera_bind_group_layout]
        };

        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{material_id:?} Pipeline Layout")),
            bind_group_layouts: &all_layouts,
            push_constant_ranges: &[],
        })
    }

    pub fn load_shader_module_descriptor(
        shader_path: &str,
    ) -> Result<wgpu::ShaderModuleDescriptor, PipelineError> {
        let shader_string = std::fs::read_to_string(shader_path);
        match shader_string {
            Ok(shader) => Ok(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(shader)),
            }),
            Err(_) => Err(PipelineError::PathNotFound(format!(
                "Failed to read shader file at path: {shader_path}"
            ))),
        }
    }

    pub fn id(&self) -> &MaterialId {
        &self.id
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }
}
