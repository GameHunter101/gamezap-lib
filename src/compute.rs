use std::{fmt::Debug, sync::Arc};

use wgpu::util::DeviceExt;

use crate::pipeline::PipelineManager;

#[derive(Debug)]
pub enum ComputeError {
    DataNotReceived,
}

pub struct ComputeShader {
    pub pipeline: wgpu::ComputePipeline,
    pub input_buffer: wgpu::Buffer,
    pub output_buffer: Arc<wgpu::Buffer>,
    pub bind_group: wgpu::BindGroup,
    pub workgroup_counts: (u32, u32, u32),
    pub data_size: u64,
    pub passive_shader: bool,
    compute_shader_index: u32,
}

impl ComputeShader {
    fn new<T: bytemuck::Pod + bytemuck::Zeroable>(
        device: &wgpu::Device,
        shader_path: &str,
        workgroup_counts: (u32, u32, u32),
        data: T,
        compute_shader_index: u32,
        passive_shader: bool,
        output_buffer_size: u64,
    ) -> Self {
        let shader_module_descriptor = PipelineManager::load_shader_module_descriptor(shader_path);
        let shader_module = device.create_shader_module(shader_module_descriptor);

        let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("Compute shader #{compute_shader_index} input buffer").as_str()),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let data_size = std::mem::size_of_val(&data) as u64;

        let output_buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(format!("Compute shader #{compute_shader_index} output buffer").as_str()),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        }));

        let bind_group_layout =
            &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(
                    format!("Compute shader #{compute_shader_index} bind group layout").as_str(),
                ),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("Compute shader #{compute_shader_index} pipeline layout").as_str()),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(format!("Compute shader #{compute_shader_index} pipeline").as_str()),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "main",
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(format!("Compute shader #{compute_shader_index} bind group").as_str()),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            pipeline,
            input_buffer,
            output_buffer,
            bind_group,
            workgroup_counts,
            data_size,
            passive_shader,
            compute_shader_index,
        }
    }

    pub async fn run_passive(&self, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!(
                "Compute shader #{} encoder",
                self.compute_shader_index
            )),
        });
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute pass"),
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
    pub async fn run<O: bytemuck::Pod + Debug + std::marker::Send>(
        &self,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
    ) -> Result<Vec<O>, ComputeError> {
        self.run_passive(device.clone(), queue.clone()).await;

        let buf = self.output_buffer.clone();
        let device = device.clone();

        let future = tokio::task::spawn(async move {
            let buffer_slice = buf.slice(..);
            let (sender, receiver) = flume::bounded(1);
            buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

            device.poll(wgpu::Maintain::Wait);

            if let Ok(Ok(())) = receiver.recv() {
                let data = buffer_slice.get_mapped_range();

                let result: Vec<O> = bytemuck::cast_slice(&data).to_vec();

                drop(data);
                buf.unmap();
                return Ok(result);
            }
            return Err(ComputeError::DataNotReceived);
        })
        .await
        .unwrap();
        return future;
    }
}

pub struct ComputeManager {
    pub shaders: Vec<ComputeShader>,
}

impl ComputeManager {
    pub fn new(shaders: Vec<ComputeShader>) -> Self {
        Self { shaders }
    }

    pub fn init() -> Self {
        Self { shaders: vec![] }
    }

    pub fn create_shader<'a, T: bytemuck::Pod + bytemuck::Zeroable>(
        &'a mut self,
        device: &wgpu::Device,
        shader_path: &str,
        workgroup_counts: (u32, u32, u32),
        data: T,
        passive_shader: bool,
    ) -> Option<&'a ComputeShader> {
        let shader = ComputeShader::new(
            device,
            shader_path,
            workgroup_counts,
            data,
            self.shaders.len() as u32,
            passive_shader,
            std::mem::size_of_val(&data) as u64,
        );
        self.shaders.push(shader);
        self.shaders.last()
    }

    pub async fn run_all_shaders(&self, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) {
        for shader in &self.shaders {
            shader.run_passive(device.clone(), queue.clone()).await;
        }
    }

    pub async fn run_passive_shaders(&self, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) {
        for shader in &self.shaders {
            if shader.passive_shader {
                shader.run_passive(device.clone(), queue.clone()).await;
            }
        }
    }
}

impl Default for ComputeManager {
    fn default() -> Self {
        Self { shaders: vec![] }
    }
}
