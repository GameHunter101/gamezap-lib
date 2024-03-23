#![allow(unused)]
use std::{num::NonZeroU32, sync::Arc};

use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType, Device,
    SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension,
};

use crate::texture::Texture;

pub type MaterialId = (String, String, usize);

#[derive(Debug)]
pub struct Material {
    vertex_shader_path: String,
    fragment_shader_path: String,
    textures: Vec<Texture>,
    enabled: bool,
    id: MaterialId,
    bind_group: BindGroup,
}

impl Material {
    pub fn new(
        vertex_shader_path: &str,
        fragment_shader_path: &str,
        textures: Vec<Texture>,
        enabled: bool,
        device: Arc<Device>,
    ) -> Self {
        let id = (
            vertex_shader_path.to_string(),
            fragment_shader_path.to_string(),
            textures.len(),
        );
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
            label: Some(&format!("Material {id:?} Bind Group Layout")),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Material {id:?} Bind Group")),
            layout: &bind_group_layout,
            entries: &bind_group_entries,
        });
        Self {
            vertex_shader_path: vertex_shader_path.to_string(),
            fragment_shader_path: fragment_shader_path.to_string(),
            textures,
            enabled,
            id,
            bind_group,
        }
    }

    pub fn id(&self) -> &MaterialId {
        &self.id
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
