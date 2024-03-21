use std::{num::NonZeroU32, sync::Arc};
use wgpu::{util::DeviceExt, Device, PipelineLayout, RenderPipeline, ShaderStages};

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
        device: Arc<wgpu::Device>,
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
        });

        Pipeline {
            pipeline: render_pipeline,
            id: id.clone(),
        }
    }

    pub fn create_pipeline_layout(material_id: &MaterialId, device: Arc<Device>) -> PipelineLayout {
        let bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry> = if material_id.2 == 0 {
            Vec::new()
        } else {
            vec![
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(material_id.2 as u32),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: NonZeroU32::new(material_id.2 as u32),
                },
            ]
        };
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{material_id:?} Bind Group Layout")),
            entries: &bind_group_layout_entries,
        });
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{material_id:?} Pipeline Layout")),
            bind_group_layouts: &[
                &bind_group_layout,
                &CameraComponent::camera_bind_group_layout(device.clone()),
            ],
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

pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub input_buffer: wgpu::Buffer,
    pub output_buffer: wgpu::Buffer,
    pub workgroup_counts: (u32, u32, u32),
    pub data_size: u64,
}

impl ComputePipeline {
    pub fn new<T: bytemuck::Pod + bytemuck::Zeroable>(
        device: &wgpu::Device,
        shader_module_descriptor: wgpu::ShaderModuleDescriptor,
        data: T,
        compute_shader_index: usize,
        workgroup_counts: (u32, u32, u32),
    ) -> Self {
        let shader_module = device.create_shader_module(shader_module_descriptor);

        let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("Compute shader #{} input buffer", compute_shader_index).as_str()),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let data_size = std::mem::size_of_val(&data) as u64;

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(format!("Compute shader #{} output buffer", compute_shader_index).as_str()),
            size: data_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(format!("Compute shader #{} pipeline", compute_shader_index).as_str()),
            layout: None,
            module: &shader_module,
            entry_point: "main",
        });

        let bind_group_layout = pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(format!("Compute shader #{} bind group", compute_shader_index).as_str()),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            }],
        });

        ComputePipeline {
            pipeline,
            bind_group,
            input_buffer,
            output_buffer,
            workgroup_counts,
            data_size,
        }
    }
}
