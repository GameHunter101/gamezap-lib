use crate::{
    camera::Camera,
    materials::{Material, MaterialManager},
    model::{Mesh, ModelVertex, Vertex},
    pipeline::{MaterialMeshGroup, Pipeline},
    renderer::Renderer,
    texture::Texture,
};

pub struct PipelineManager<'a> {
    pub material_mesh_groups: Vec<MaterialMeshGroup<'a>>,
    pub no_texture_pipeline: Option<Pipeline>,
}

impl<'a> PipelineManager<'a> {
    pub fn init() -> Self {
        PipelineManager {
            material_mesh_groups: vec![],
            no_texture_pipeline: None,
        }
    }

    pub fn create_pipelines(
        &mut self,
        material_manager: &MaterialManager,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        camera: &Camera,
    ) {
        for material in &material_manager.materials {
            match material.pipeline_type {
                PipelineType::NoTextures => {
                    if self.no_texture_pipeline.is_none() {
                        let pipeline_layout = device.create_pipeline_layout(
                            &wgpu::PipelineLayoutDescriptor {
                                label: Some("NoTexturePipelineLayout"),
                                bind_group_layouts: &[
                                    &material.bind_group_layout,
                                    &camera.bind_group_layout,
                                ],
                                push_constant_ranges: &[],
                            },
                        );

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
                _ => {}
            }
        }
    }

    // pub fn new_group(
    //     &mut self,
    //     material: Material,
    //     meshes: Vec<Mesh>,
    //     vertex_shader: wgpu::ShaderModuleDescriptor,
    //     fragment_shader: wgpu::ShaderModuleDescriptor,
    // ) {
    //     let group = MaterialMeshGroup::new(
    //         material,
    //         meshes,
    //         self.renderer,
    //         vertex_shader,
    //         fragment_shader,
    //     );
    // }
}

pub enum PipelineType {
    NoTextures,
    DiffuseTexture,
    NormalDiffuseTexture,
}
