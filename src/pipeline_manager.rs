use crate::{
    model::{Material, Mesh},
    pipeline::MaterialMeshGroup,
    renderer::Renderer,
};

pub struct PipelineManager<'a> {
    pub material_mesh_groups: Vec<MaterialMeshGroup<'a>>,
    pub renderer: &'a Renderer<'a>,
}

impl<'a> PipelineManager<'a> {
    pub fn init(renderer: &'a Renderer) -> Self {
        PipelineManager {
            material_mesh_groups: vec![],
            renderer,
        }
    }

    pub fn new_group(
        &mut self,
        material: Material,
        meshes: Vec<Mesh>,
        vertex_shader: wgpu::ShaderModuleDescriptor,
        fragment_shader: wgpu::ShaderModuleDescriptor,
    ) {
        let group = MaterialMeshGroup::new(
            material,
            meshes,
            self.renderer,
            vertex_shader,
            fragment_shader,
        );
    }
}
