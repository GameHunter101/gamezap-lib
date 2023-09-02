use crate::{model::Mesh, texture::Texture, pipeline::PipelineType};

#[derive(Debug)]
pub struct MaterialManager {
    pub plain_materials: Vec<Material>,
    pub diffuse_texture_materials: Vec<Material>,
    pub diffuse_normal_texture_materials: Vec<Material>,
}

impl MaterialManager {
    pub fn init() -> Self {
        MaterialManager {
            plain_materials: vec![],
            diffuse_texture_materials: vec![],
            diffuse_normal_texture_materials: vec![],
        }
    }

    pub fn new_material(
        &mut self,
        name: &str,
        device: &wgpu::Device,
        diffuse_texture: Option<Texture>,
        normal_texture: Option<Texture>,
    ) -> Material {
        let mut material_index = self.plain_materials.len() as u32;
        if diffuse_texture.is_some() {
            material_index = self.diffuse_texture_materials.len() as u32;
            if normal_texture.is_some() {
                material_index = self.diffuse_normal_texture_materials.len() as u32;
            }
        }
        let material = Material::new(
            name,
            device,
            diffuse_texture,
            normal_texture,
            material_index,
        );
        material
    }

    pub fn add_materials(&mut self, materials: Vec<Material>) {
        for material in materials {
            if material.diffuse_texture.is_some() {
                if material.normal_texture.is_some() {
                    self.diffuse_normal_texture_materials.push(material);
                    continue;
                }
                self.diffuse_texture_materials.push(material);
                continue;
            }
            self.plain_materials.push(material);
            continue;
        }
    }
}

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: Option<Texture>,
    pub normal_texture: Option<Texture>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub pipeline_type: PipelineType,
    pub meshes: Vec<Mesh>,
    pub material_index: u32,
}

impl Material {
    pub fn new(
        name: &str,
        device: &wgpu::Device,
        diffuse_texture: Option<Texture>,
        normal_texture: Option<Texture>,
        material_index: u32,
    ) -> Self {
        let mut layout_entries = vec![];
        if diffuse_texture.is_some() {
            let diffuse_entry_texture = wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            };
            let diffuse_entry_sampler = wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            };
            layout_entries.push(diffuse_entry_texture);
            layout_entries.push(diffuse_entry_sampler);

            if normal_texture.is_some() {
                let normal_entry_texture = wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                };
                let normal_entry_sampler = wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                };
                layout_entries.push(normal_entry_texture);
                layout_entries.push(normal_entry_sampler);
            }
        }
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{}_material_bind_group_layout", name)),
            entries: &layout_entries,
        });
        let mut bind_group_entries: Vec<wgpu::BindGroupEntry> = vec![];
        if let Some(diffuse_texture) = &diffuse_texture {
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
            });

            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
            });
        }
        if let Some(normal_texture) = &normal_texture {
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&normal_texture.view),
            });

            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
            });
        }
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout: &bind_group_layout,
            entries: &bind_group_entries,
        });

        let mut pipeline_type = PipelineType::Plain;
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
            bind_group_layout,
            bind_group,
            pipeline_type,
            meshes: vec![],
            material_index,
        }
    }
}
