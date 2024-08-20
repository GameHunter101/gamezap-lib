use std::{fmt::Debug, rc::Rc, sync::Arc};

use enum_as_inner::EnumAsInner;
use wgpu::{util::DeviceExt, Buffer, Device, Queue};

use crate::texture::Texture;

#[derive(Debug)]
pub enum ComputeError {
    InvalidCast,
    BufferMapError,
    AssetIsNotBuffer,
}

#[derive(Debug)]
pub struct ComputePipelineType<'a, T: bytemuck::Pod + bytemuck::Zeroable> {
    pub input_data: Vec<ComputeData<'a, T>>,
    pub output_data_type: Vec<ComputeOutput>,
}

#[derive(Debug)]
pub enum ComputeOutput {
    Array(u64),
    Texture((u32, u32)),
}

#[derive(Debug, EnumAsInner)]
pub enum ComputeData<'a, T: bytemuck::Pod + bytemuck::Zeroable> {
    ArrayData(&'a [T]),
    TextureData((ComputeTextureData, bool)),
}

#[derive(Debug)]
pub enum ComputeTextureData {
    Path(String),
    Dimensions((u32, u32)),
}

#[derive(Debug, EnumAsInner)]
pub enum ComputePackagedData {
    Buffer(Rc<Buffer>),
    Texture(Rc<Texture>),
}

#[derive(Debug)]
pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub pipeline_assets: Vec<ComputePackagedData>,
    pub workgroup_counts: (u32, u32, u32),
    pub compute_shader_index: usize,
}

impl ComputePipeline {
    pub fn new<T: bytemuck::Pod + bytemuck::Zeroable + Debug>(
        device: Arc<Device>,
        queue: Arc<Queue>,
        shader_module_descriptor: wgpu::ShaderModuleDescriptor,
        pipeline_type: ComputePipelineType<T>,
        compute_shader_index: usize,
        workgroup_counts: (u32, u32, u32),
    ) -> Self {
        let shader_module = device.create_shader_module(shader_module_descriptor);

        let (bind_group_layout, pipeline) = Self::create_bind_group_layout_and_pipeline(
            device.clone(),
            shader_module,
            &pipeline_type,
            compute_shader_index,
        );

        let pipeline_assets = Self::create_pipeline_assets(
            device.clone(),
            queue.clone(),
            &pipeline_type,
            compute_shader_index,
        );

        let bind_group_entries = Self::create_pipeline_bind_group_entries(&pipeline_assets);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!(
                "Compute shader #{} bind group",
                compute_shader_index
            )),
            layout: &bind_group_layout,
            entries: &bind_group_entries,
        });

        ComputePipeline {
            pipeline,
            bind_group_layout,
            bind_group,
            pipeline_assets,
            workgroup_counts,
            compute_shader_index,
        }
    }

    fn create_bind_group_layout_and_pipeline<T: bytemuck::Pod + bytemuck::Zeroable + Debug>(
        device: Arc<Device>,
        shader_module: wgpu::ShaderModule,
        pipeline_type: &ComputePipelineType<T>,
        compute_shader_index: usize,
    ) -> (wgpu::BindGroupLayout, wgpu::ComputePipeline) {
        let input_data = &pipeline_type.input_data;

        let input_entries = input_data.iter().enumerate().map(|(i, entry)| match entry {
            ComputeData::ArrayData(_) => Self::create_array_bind_group_layout_entry(i as u32),
            ComputeData::TextureData((_, is_write)) => {
                Self::create_texture_bind_group_layout_entry(i as u32, *is_write)
            }
        });

        let input_len = input_entries.len();

        let output_data = &pipeline_type.output_data_type;

        let output_entries = output_data
            .iter()
            .enumerate()
            .map(|(i, entry)| match entry {
                ComputeOutput::Array(_) => {
                    Self::create_array_bind_group_layout_entry((input_len + i) as u32)
                }
                ComputeOutput::Texture(_) => {
                    Self::create_texture_bind_group_layout_entry((input_len + i) as u32, true)
                }
            });

        let entries = input_entries.chain(output_entries).collect::<Vec<_>>();

        let pipeline_bind_group_layouts =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!(
                    "Compute Shader #{compute_shader_index} group layout"
                )),
                entries: &entries,
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

        (pipeline_bind_group_layouts, pipeline)
    }

    fn create_pipeline_assets<T: bytemuck::Pod + bytemuck::Zeroable + Debug>(
        device: Arc<Device>,
        queue: Arc<Queue>,
        pipeline_type: &ComputePipelineType<T>,
        compute_shader_index: usize,
    ) -> Vec<ComputePackagedData> {
        let input_data = &pipeline_type.input_data;

        let packaged_input_data = input_data.iter().enumerate().map(|(i, entry)| match entry {
            ComputeData::ArrayData(arr) => ComputePackagedData::Buffer(Rc::new(
                Self::create_array_buffer(device.clone(), arr, compute_shader_index, i),
            )),
            ComputeData::TextureData((tex_data, _)) => {
                ComputePackagedData::Texture(Rc::new(match tex_data {
                    ComputeTextureData::Path(path) => pollster::block_on(Texture::load_texture(
                        path, false, &device, &queue, false,
                    ))
                    .unwrap(),
                    ComputeTextureData::Dimensions((width, height)) => Texture::blank_texture(
                        &device.clone(),
                        &queue.clone(),
                        *width,
                        *height,
                        Some(&format!(
                            "Compute shader #{compute_shader_index} input asset #{i} (texture)"
                        )),
                        true,
                    )
                    .unwrap(),
                }))
            }
        });

        let output_data = &pipeline_type.output_data_type;

        let packaged_output_data = output_data
            .iter()
            .enumerate()
            .map(|(i, entry)| match entry {
                ComputeOutput::Array(buf_size) => ComputePackagedData::Buffer(Rc::new(
                    device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(
                            format!(
                                "Compute shader #{compute_shader_index} output asset #{i} (buffer)"
                            )
                            .as_str(),
                        ),
                        size: *buf_size,
                        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::STORAGE,
                        mapped_at_creation: false,
                    }),
                )),
                ComputeOutput::Texture((width, height)) => ComputePackagedData::Texture(Rc::new(
                    Texture::blank_texture(
                        &device.clone(),
                        &queue.clone(),
                        *width,
                        *height,
                        Some(
                            format!(
                                "Compute shader #{compute_shader_index} output asset #{i} (buffer)"
                            )
                            .as_str(),
                        ),
                        true,
                    )
                    .unwrap(),
                )),
            });

        packaged_input_data.chain(packaged_output_data).collect()
    }

    fn create_pipeline_bind_group_entries(
        pipeline_assets: &[ComputePackagedData],
    ) -> Vec<wgpu::BindGroupEntry> {
        pipeline_assets
            .iter()
            .enumerate()
            .map(|(i, entry)| match entry {
                ComputePackagedData::Buffer(buf) => wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: buf.as_entire_binding(),
                },
                ComputePackagedData::Texture(tex) => wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: wgpu::BindingResource::TextureView(&tex.view),
                },
            })
            .collect()
    }

    pub fn update_pipeline_assets(
        &mut self,
        device: Arc<Device>,
        new_assets: Vec<(ComputePackagedData, usize)>,
    ) {
        for (new_asset, index) in new_assets.into_iter() {
            self.pipeline_assets[index] = new_asset;
        }

        let bind_group_entries = Self::create_pipeline_bind_group_entries(&self.pipeline_assets);

        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!(
                "Compute shader #{} bind group",
                self.compute_shader_index,
            )),
            layout: &self.bind_group_layout,
            entries: &bind_group_entries,
        });
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
        is_write: bool,
    ) -> wgpu::BindGroupLayoutEntry {
        if is_write {
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadWrite,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }
        } else {
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
        }
    }

    fn create_array_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
        device: Arc<Device>,
        arr: &[T],
        compute_shader_index: usize,
        buffer_id: usize,
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(
                format!("Compute shader #{compute_shader_index} input array buffer #{buffer_id}",)
                    .as_str(),
            ),
            contents: bytemuck::cast_slice(arr),
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

    pub fn run_compute_shader(&self, device: &Device, queue: &Queue) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!(
                "Compute shader #{} encoder",
                self.compute_shader_index
            )),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(&format!(
                    "Compute shader #{} compute pass",
                    self.compute_shader_index
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
    }

    pub fn grab_array_data<
        T: bytemuck::Pod + bytemuck::Zeroable + std::marker::Sync + std::marker::Send,
    >(
        &self,
        device: Arc<Device>,
        asset_index: usize,
    ) -> Result<Vec<T>, ComputeError> {
        if let ComputePackagedData::Buffer(buf) = &self.pipeline_assets[asset_index] {
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
                    Ok(vec)
                } else {
                    Err(ComputeError::InvalidCast)
                }
            } else {
                Err(ComputeError::BufferMapError)
            }
        } else {
            Err(ComputeError::AssetIsNotBuffer)
        }
    }
}
