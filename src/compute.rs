use wgpu::util::DeviceExt;

use crate::pipeline::PipelineManager;

pub struct ComputeShader {
    pub pipeline: wgpu::ComputePipeline,
    pub input_buffer: wgpu::Buffer,
    pub output_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub workgroup_counts: (u32, u32, u32),
    pub data_size: u64,
}

impl ComputeShader {
    fn new<T: bytemuck::Pod + bytemuck::Zeroable>(
        device: &wgpu::Device,
        shader_path: &str,
        workgroup_counts: (u32, u32, u32),
        data: T,
        compute_shader_index: u32,
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

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(format!("Compute shader #{compute_shader_index} output buffer").as_str()),
            size: data_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout =
            &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(
                    format!("Compute shader #{compute_shader_index} bind group layout").as_str(),
                ),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
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
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            input_buffer,
            output_buffer,
            bind_group,
            workgroup_counts,
            data_size,
        }
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
    ) -> Option<&'a ComputeShader> {
        let test = ComputeShader::new(
            device,
            shader_path,
            workgroup_counts,
            data,
            self.shaders.len() as u32,
        );
        self.shaders.push(test);
        self.shaders.last()
    }

    pub async fn run_shaders(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for (i, compute_shader) in self.shaders.iter().enumerate() {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("Compute shader #{i} encoder")),
            });
            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute pass"),
                });

                compute_pass.set_pipeline(&compute_shader.pipeline);
                compute_pass.set_bind_group(0, &compute_shader.bind_group, &[]);
                compute_pass.dispatch_workgroups(
                    compute_shader.workgroup_counts.0,
                    compute_shader.workgroup_counts.1,
                    compute_shader.workgroup_counts.2,
                );
            }
            encoder.copy_buffer_to_buffer(
                &compute_shader.input_buffer,
                0,
                &compute_shader.output_buffer,
                0,
                compute_shader.data_size,
            );

            queue.submit(Some(encoder.finish()));

            let buffer_slice = compute_shader.output_buffer.slice(..);

            let (sender, receiver) = flume::bounded(1);
            buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

            // TODO: Make this not block
            device.poll(wgpu::Maintain::Wait);

            if let Ok(Ok(())) = receiver.recv_async().await {
                let data = buffer_slice.get_mapped_range();

                let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
                println!("Compute #{i} result: {result:?}");

                drop(data);
                compute_shader.output_buffer.unmap();
            }
        }
    }
}

impl Default for ComputeManager {
    fn default() -> Self {
        Self { shaders: vec![] }
    }
}
