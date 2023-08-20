use crate::{pipeline_manager::PipelineType, texture::Texture};

pub struct MaterialManager {
    pub materials: Vec<Material>,
}

impl MaterialManager {
    pub fn init() -> Self {
        MaterialManager { materials: vec![] }
    }
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: Option<Texture>,
    pub normal_texture: Option<Texture>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub pipeline_type: PipelineType,
}

impl Material {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        diffuse_texture: Option<Texture>,
        normal_texture: Option<Texture>,
        layout: wgpu::BindGroupLayout,
    ) -> Self {
        let mut entries: Vec<wgpu::BindGroupEntry> = vec![];
        if let Some(diffuse_texture) = &diffuse_texture {
            entries.push(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
            });

            entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
            });
        }
        if let Some(normal_texture) = &normal_texture {
            entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&normal_texture.view),
            });

            entries.push(wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
            });
        }
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout: &layout,
            entries: &entries,
        });

        let mut pipeline_type = PipelineType::NoTextures;
        if diffuse_texture.is_some() {
            if normal_texture.is_some() {
                pipeline_type = PipelineType::NormalDiffuseTexture;
            } else {
                pipeline_type = PipelineType::DiffuseTexture;
            }
        }

        Material {
            name: String::from(name),
            diffuse_texture,
            normal_texture,
            bind_group_layout: layout,
            bind_group,
            pipeline_type,
        }
    }
}
