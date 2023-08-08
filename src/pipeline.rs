use crate::{
    model::{Material, Mesh, ModelVertex, Vertex},
    renderer::Renderer,
    texture::Texture,
};

pub struct MaterialMeshGroup<'a> {
    pub material: Material,
    pub meshes: Vec<Mesh>,
    pub pipeline: Pipeline,
    pub camera_bind_group: &'a wgpu::BindGroup,
    pub num_indices: u32,
}

impl<'a> MaterialMeshGroup<'a> {
    pub fn new(
        material: Material,
        meshes: Vec<Mesh>,
        renderer: &'a Renderer,
        vertex_shader: wgpu::ShaderModuleDescriptor,
        fragment_shader: wgpu::ShaderModuleDescriptor,
    ) -> Self {
        let camera = renderer.camera.unwrap();
        let pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(&format!("{} pipeline layout", material.name)),
                    bind_group_layouts: &[&material.bind_group_layout, &camera.bind_group_layout],
                    push_constant_ranges: &[],
                });
        let pipeline = Pipeline::new(
            &renderer.device,
            &pipeline_layout,
            renderer.config.format,
            Some(Texture::DEPTH_FORMAT),
            &[ModelVertex::desc()],
            vertex_shader,
            fragment_shader,
        );
        let mut num_indices = 0;
        for mesh in &meshes {
            num_indices += mesh.num_indices;
        }
        MaterialMeshGroup {
            material,
            meshes,
            pipeline,
            camera_bind_group: &camera.bind_group,
            num_indices,
        }
    }
}

pub struct Pipeline {
    pub pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
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
            label: Some("Render pipeline"),
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
                    blend: Some(wgpu::BlendState::REPLACE),
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
