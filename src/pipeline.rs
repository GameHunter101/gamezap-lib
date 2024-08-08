#![allow(clippy::too_many_arguments)]
use enum_as_inner::EnumAsInner;
use std::{fmt::Debug, num::NonZeroU32, sync::Arc};
use wgpu::{util::DeviceExt, Buffer, Device, PipelineLayout, Queue, RenderPipeline, ShaderStages};

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
        let texture_bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry> =
            if material_id.2 == 0 {
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

#[derive(Debug)]
pub enum ComputeError {
    InvalidCast,
    BufferMapError,
}

#[derive(Debug)]
pub struct ComputePipelineType<T: bytemuck::Pod + bytemuck::Zeroable> {
    pub input_data: ComputeData<T>,
    pub output_data_type: ComputeOutput,
}

#[derive(Debug)]
pub enum ComputeOutput {
    Array(u64),
    Texture((u32, u32)),
}

#[derive(Debug, EnumAsInner)]
pub enum ComputeData<T: bytemuck::Pod + bytemuck::Zeroable> {
    ArrayData(T),
    TextureData(ComputeTextureData),
}

#[derive(Debug)]
pub enum ComputeTextureData {
    Path(String),
    Dimensions((u32, u32)),
}

#[derive(Debug, EnumAsInner)]
pub enum ComputePackagedData {
    Buffer(Arc<Buffer>),
    Texture(Texture),
}

#[derive(Debug)]
pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub output_data: ComputePackagedData,
    pub workgroup_counts: (u32, u32, u32),
    pub data_size: Option<u64>,
    pub pipeline_id: usize,
}

impl ComputePipeline {
    pub fn new<T: bytemuck::Pod + bytemuck::Zeroable + Debug>(
        device: Arc<Device>,
        queue: Arc<Queue>,
        shader_module_descriptor: wgpu::ShaderModuleDescriptor,
        pipeline_type: ComputePipelineType<T>,
        compute_shader_index: usize,
        workgroup_counts: (u32, u32, u32),
        pipeline_id: usize,
    ) -> Self {
        let data = pipeline_type.input_data;

        let shader_module = device.create_shader_module(shader_module_descriptor);

        let pipeline_bind_group_layouts =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!(
                    "Compute Shader #{compute_shader_index} group layout"
                )),
                entries: &[
                    match data {
                        ComputeData::ArrayData(_) => Self::create_array_bind_group_layout_entry(0),
                        ComputeData::TextureData(_) => {
                            Self::create_texture_bind_group_layout_entry(0, true)
                        }
                    },
                    match pipeline_type.output_data_type {
                        ComputeOutput::Array(_) => Self::create_array_bind_group_layout_entry(1),
                        ComputeOutput::Texture((_, _)) => {
                            Self::create_texture_bind_group_layout_entry(1, false)
                        }
                    },
                ],
            });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(format!("Compute shader #{} pipeline", compute_shader_index).as_str()),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!(
                        "Compute shader #{compute_shader_index} pipeline layout"
                    )),
                    bind_group_layouts: &[&pipeline_bind_group_layouts],
                    push_constant_ranges: &[],
                }),
            ),
            module: &shader_module,
            entry_point: "main",
        });

        let output_data = match &pipeline_type.output_data_type {
            ComputeOutput::Array(data_size) => ComputePackagedData::Buffer(Arc::new(
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(
                        format!("Compute shader #{} output buffer", compute_shader_index).as_str(),
                    ),
                    size: *data_size,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }),
            )),
            ComputeOutput::Texture((width, height)) => ComputePackagedData::Texture({
                Texture::blank_texture(
                    &device,
                    &queue,
                    *width,
                    *height,
                    Some(&format!("Compute shader #{pipeline_id} output texture")),
                    true,
                )
                .unwrap()
            }),
        };

        let input_texture = match data {
            ComputeData::ArrayData(arr) => ComputePackagedData::Buffer(Arc::new(
                Self::create_array_buffer(device.clone(), arr, compute_shader_index),
            )),
            ComputeData::TextureData(ref tex_data) => {
                ComputePackagedData::Texture(match tex_data {
                    ComputeTextureData::Path(path) => pollster::block_on(Texture::load_texture(
                        path, false, &device, &queue, false,
                    ))
                    .unwrap(),
                    ComputeTextureData::Dimensions((width, height)) => Texture::blank_texture(
                        &device,
                        &queue,
                        *width,
                        *height,
                        Some(&format!("Compute shader #{pipeline_id} input texture")),
                        true,
                    )
                    .unwrap(),
                })
            }
        };

        let input_bind_group_entry = match &data {
            ComputeData::ArrayData(_) => wgpu::BindGroupEntry {
                binding: 0,
                resource: input_texture.as_buffer().unwrap().as_entire_binding(),
            },
            ComputeData::TextureData(_) => wgpu::BindGroupEntry {
                binding: 0,
                resource: {
                    wgpu::BindingResource::TextureView(&input_texture.as_texture().unwrap().view)
                },
            },
        };

        let output_bind_group_entry = match &output_data {
            ComputePackagedData::Buffer(buf) => wgpu::BindGroupEntry {
                binding: 1,
                resource: buf.as_entire_binding(),
            },
            ComputePackagedData::Texture(tex) => wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&tex.view),
            },
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!(
                "Compute shader #{} bind group",
                compute_shader_index
            )),
            layout: &pipeline_bind_group_layouts,
            entries: &[input_bind_group_entry, output_bind_group_entry],
        });

        /* let bind_group = match &output_data {
            ComputePackagedData::Buffer(output_buffer) => {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(
                        format!("Compute shader #{} bind group", compute_shader_index).as_str(),
                    ),
                    layout: &pipeline_bind_group_layouts,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: input_buffer.unwrap().as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: output_buffer.as_entire_binding(),
                        },
                    ],
                })
            }
            ComputePackagedData::Texture(output_texture) => {
                let input_texture = match data.as_texture_data().unwrap() {
                    ComputeTextureData::Path(path) => pollster::block_on(Texture::load_texture(
                        path, false, &device, &queue, false,
                    ))
                    .unwrap(),
                    ComputeTextureData::Dimensions((width, height)) => Texture::blank_texture(
                        &device,
                        &queue,
                        *width,
                        *height,
                        Some(&format!("Compute shader #{pipeline_id} input texture")),
                        true,
                    )
                    .unwrap(),
                };
                Self::create_texture_bind_group(
                    device,
                    &[&input_texture, output_texture],
                    pipeline_id,
                )
            }
        }; */

        ComputePipeline {
            pipeline,
            bind_group,
            output_data,
            workgroup_counts,
            data_size: match pipeline_type.output_data_type {
                ComputeOutput::Array(size) => Some(size),
                ComputeOutput::Texture(_) => None,
            },
            pipeline_id,
        }
    }

    fn create_array_bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn create_texture_bind_group_layout_entry(
        binding: u32,
        is_input: bool,
    ) -> wgpu::BindGroupLayoutEntry {
        if is_input {
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }
        } else {
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }
        }
    }

    fn create_array_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
        device: Arc<Device>,
        arr: T,
        compute_shader_index: usize,
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(
                format!(
                    "Compute shader #{} input array buffer",
                    compute_shader_index
                )
                .as_str(),
            ),
            contents: bytemuck::cast_slice(&[arr]),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        })
    }

    pub fn create_texture_bind_group(
        device: Arc<Device>,
        textures: &[&Texture],
        pipeline_id: usize,
    ) -> wgpu::BindGroup {
        let bind_group_layout_entries = vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ];

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("Compute shader #{pipeline_id} bind group layout")),
            entries: &bind_group_layout_entries,
        });

        let bind_group_entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&textures[0].view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&textures[1].view),
            },
        ];

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute shader #{pipeline_id} texture bind group"),
            layout: &bind_group_layout,
            entries: &bind_group_entries,
        })
    }

    pub fn run_compute_shader<
        T: bytemuck::Pod + bytemuck::Zeroable + std::marker::Sync + std::marker::Send,
    >(
        &self,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> Result<Option<Vec<T>>, ComputeError> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("Compute shader #{} encoder", self.pipeline_id)),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(&format!(
                    "Compute shader #{} compute pass",
                    self.pipeline_id
                )),
            });

            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(
                self.workgroup_counts.0,
                self.workgroup_counts.1,
                self.workgroup_counts.2,
            );
        }

        queue.submit(Some(encoder.finish()));

        if let ComputePackagedData::Buffer(buf) = &self.output_data {
            let buf = buf.clone();

            let buffer_slice = buf.slice(..);
            let (sender, receiver) = flume::bounded(1);
            buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

            device.poll(wgpu::Maintain::Wait);

            if let Ok(Ok(())) = receiver.recv() {
                let data_buffer = buffer_slice.get_mapped_range();

                let data_result: Result<&[T], bytemuck::PodCastError> =
                    bytemuck::try_cast_slice(&data_buffer);

                if let Ok(result) = data_result {
                    let vec = result.to_vec();
                    drop(data_buffer);
                    buf.unmap();
                    return Ok(Some(vec));
                } else {
                    return Err(ComputeError::InvalidCast);
                }
            } else {
                return Err(ComputeError::BufferMapError);
            }
        }
        Ok(None)
    }
}
