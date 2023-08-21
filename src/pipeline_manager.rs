use crate::{
    camera::Camera,
    materials::MaterialManager,
    model::{ModelVertex, Vertex},
    pipeline::Pipeline,
    texture::Texture,
};

pub struct PipelineManager {
    pub materials: MaterialManager,
    pub no_texture_pipeline: Option<Pipeline>,
    pub diffuse_texture_pipeline: Option<Pipeline>,
    pub diffuse_normal_texture_pipeline: Option<Pipeline>,
}

impl PipelineManager {
    pub fn init() -> Self {
        PipelineManager {
            materials: MaterialManager::init(),
            no_texture_pipeline: None,
            diffuse_texture_pipeline: None,
            diffuse_normal_texture_pipeline: None,
        }
    }

    pub fn create_pipelines(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        camera: &Camera,
    ) {
        if self.materials.no_texture_materials.len() > 0 {
            if self.no_texture_pipeline.is_none() {
                let pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("NoTexturePipelineLayout"),
                        bind_group_layouts: &[
                            &self.materials.no_texture_materials[0].bind_group_layout,
                            &camera.bind_group_layout,
                        ],
                        push_constant_ranges: &[],
                    });

                let vertex_shader = wgpu::include_wgsl!("../examples/shaders/vert.wgsl");
                let fragment_shader = wgpu::include_wgsl!("../examples/shaders/frag.wgsl");

                self.no_texture_pipeline = Some(Pipeline::new(
                    device,
                    &pipeline_layout,
                    format,
                    Some(Texture::DEPTH_FORMAT),
                    &[ModelVertex::desc()],
                    vertex_shader,
                    fragment_shader,
                ));
            }
        }
    }
}

pub enum PipelineType {
    NoTextures,
    DiffuseTexture,
    NormalDiffuseTexture,
}
