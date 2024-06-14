#![allow(unused)]
use std::{num::NonZeroU32, sync::Arc};

use wgpu::{
    util::DeviceExt, BindGroup, BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType,
    Device, SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension,
};

use crate::texture::Texture;

pub type MaterialId = (String, String, usize, bool);

#[derive(Debug)]
pub struct Material {
    vertex_shader_path: String,
    fragment_shader_path: String,
    textures: Vec<Texture>,
    enabled: bool,
    id: MaterialId,
    texture_bind_group: BindGroup,
    uniform_buffer_bind_group: Option<BindGroup>,
}

impl Material {
    pub fn new(
        vertex_shader_path: &str,
        fragment_shader_path: &str,
        textures: Vec<Texture>,
        uniform_buffer_data: Option<&[u8]>,
        enabled: bool,
        device: Arc<Device>,
    ) -> Self {
        let id = (
            vertex_shader_path.to_string(),
            fragment_shader_path.to_string(),
            textures.len(),
            uniform_buffer_data.is_some(),
        );

        let texture_bind_group =
            Self::create_texture_bind_group(&textures, device.clone(), id.clone());

        let uniform_buffer_bind_group = uniform_buffer_data
            .map(|data| Self::create_uniform_buffer_bind_group(id.clone(), device, data));

        Self {
            vertex_shader_path: vertex_shader_path.to_string(),
            fragment_shader_path: fragment_shader_path.to_string(),
            textures,
            enabled,
            id,
            texture_bind_group,
            uniform_buffer_bind_group,
        }
    }

    fn create_texture_bind_group(
        textures: &[Texture],
        device: Arc<Device>,
        material_id: MaterialId,
    ) -> BindGroup {
        let bind_group_layout_entries = if textures.is_empty() {
            Vec::new()
        } else {
            vec![
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(textures.len() as u32),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: NonZeroU32::new(textures.len() as u32),
                },
            ]
        };
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("Material {material_id:?} Bind Group Layout")),
            entries: &bind_group_layout_entries,
        });
        let views = textures.iter().map(|tex| &tex.view).collect::<Vec<_>>();

        let samplers = textures.iter().map(|tex| &tex.sampler).collect::<Vec<_>>();

        let bind_group_entries = if textures.is_empty() {
            Vec::new()
        } else {
            vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureViewArray(&views),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::SamplerArray(&samplers),
                },
            ]
        };

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Material {material_id:?} Texture Bind Group")),
            layout: &bind_group_layout,
            entries: &bind_group_entries,
        })
    }

    fn create_uniform_buffer_bind_group(
        material_id: MaterialId,
        device: Arc<Device>,
        data: &[u8],
    ) -> BindGroup {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("Material {material_id:?} Bind Group")),
            entries: &[BindGroupLayoutEntry {
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

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Material {material_id:?} Uniform Buffer")),
            contents: data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_entries = [BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }];

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!(
                "Material {material_id:?} Uniform Buffer Bind Group"
            )),
            layout: &bind_group_layout,
            entries: &bind_group_entries,
        })
    }

    pub fn id(&self) -> &MaterialId {
        &self.id
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn texture_bind_group(&self) -> &BindGroup {
        &self.texture_bind_group
    }

    pub fn uniform_buffer_bind_group(&self) -> Option<&BindGroup> {
        self.uniform_buffer_bind_group.as_ref()
    }
}
