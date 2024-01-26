use std::{cell::Ref, num::NonZeroU32, sync::Arc};
use wgpu::{util::DeviceExt, Device, PipelineLayout, ShaderStages};

use crate::{
    camera::CameraManager,
    ecs::component::MaterialId,
    materials::MaterialManager,
    model::{Mesh, Vertex, VertexData},
    texture::Texture,
};

pub struct PipelineManager {
    pub plain_pipeline: Option<Pipeline>,
    pub diffuse_texture_pipeline: Option<Pipeline>,
    pub diffuse_normal_texture_pipeline: Option<Pipeline>,
    pub compute_shaders: Vec<ComputePipeline>,
}

impl PipelineManager {
    pub fn init() -> Self {
        PipelineManager {
            plain_pipeline: None,
            diffuse_texture_pipeline: None,
            diffuse_normal_texture_pipeline: None,
            compute_shaders: vec![],
        }
    }

    pub fn create_pipelines(
        &mut self,
        device: Arc<wgpu::Device>,
        format: wgpu::TextureFormat,
        material_manager: Ref<MaterialManager>,
        camera_manager: Option<Ref<CameraManager>>,
    ) {
        /* if material_manager.plain_materials.len() > 0 {
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
                    device
                        .clone()
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
        } */
    }

    pub fn create_compute_shader<T: bytemuck::Pod + bytemuck::Zeroable>(
        &mut self,
        device: &wgpu::Device,
        shader_name: &str,
        data: T,
        workgroup_counts: (u32, u32, u32),
    ) {
        let shader_module_descriptor = PipelineManager::load_shader_module_descriptor(shader_name);
        self.compute_shaders.push(ComputePipeline::new(
            device,
            shader_module_descriptor,
            data,
            self.compute_shaders.len(),
            workgroup_counts,
        ));
    }

    pub fn load_shader_module_descriptor(shader_path: &str) -> wgpu::ShaderModuleDescriptor {
        let shader_string =
            std::fs::read_to_string(shader_path).expect("Failed to read shader file");

        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(shader_string)),
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
        device: Arc<wgpu::Device>,
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

    pub fn create_pipeline_layout(material_id: &MaterialId, device: Arc<Device>) -> PipelineLayout {
        let bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry> = vec![
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
        ];
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{material_id:?} Bind Group Layout")),
            entries: &bind_group_layout_entries,
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{material_id:?} Pipeline Layout")),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        layout
    }

    pub fn load_shader_module_descriptor(shader_path: &str) -> wgpu::ShaderModuleDescriptor {
        let shader_string =
            std::fs::read_to_string(shader_path).expect("Failed to read shader file");

        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(shader_string)),
        }
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
