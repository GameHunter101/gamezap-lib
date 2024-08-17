#![allow(unused)]
use std::{num::NonZeroU32, sync::Arc};

use wgpu::{
    util::DeviceExt, BindGroup, BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType,
    Buffer, Device, SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension,
};

use crate::texture::Texture;

pub type MaterialId = (String, String, usize, bool);

#[derive(Debug)]
pub struct Material<'a> {
    vertex_shader_path: String,
    fragment_shader_path: String,
    views_and_samplers: Vec<(&'a wgpu::TextureView, &'a wgpu::Sampler)>,
    enabled: bool,
    id: MaterialId,
    texture_bind_group: BindGroup,
    uniform_buffer_and_bind_group: Option<(BindGroup, Buffer)>,
}

impl<'a> Material<'a> {
    pub fn new(
        vertex_shader_path: &str,
        fragment_shader_path: &str,
        textures: &'a [Texture],
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

        let views_and_samplers = textures
            .iter()
            .map(|tex| (&tex.view, &tex.sampler))
            .collect::<Vec<_>>();

        let texture_bind_group =
            Self::create_texture_bind_group(&views_and_samplers, device.clone());

        let uniform_buffer_and_bind_group = uniform_buffer_data
            .map(|data| Self::create_uniform_buffer_and_bind_group(id.clone(), device, data));

        Self {
            vertex_shader_path: vertex_shader_path.to_string(),
            fragment_shader_path: fragment_shader_path.to_string(),
            views_and_samplers,
            enabled,
            id,
            texture_bind_group,
            uniform_buffer_and_bind_group,
        }
    }

    pub fn create_texture_bind_group(
        views_and_samplers: &[(&wgpu::TextureView, &wgpu::Sampler)],
        device: Arc<Device>,
    ) -> BindGroup {
        let bind_group_layout_entries = if views_and_samplers.is_empty() {
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
                    count: NonZeroU32::new(views_and_samplers.len() as u32),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: NonZeroU32::new(views_and_samplers.len() as u32),
                },
            ]
        };
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &bind_group_layout_entries,
        });

        let mut views = Vec::new();
        let mut samplers = Vec::new();
        for (view, sampler) in views_and_samplers {
            views.push(*view);
            samplers.push(*sampler);
        }

        let bind_group_entries = if views.is_empty() {
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
            label: Some("Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &bind_group_entries,
        })
    }

    fn create_uniform_buffer_and_bind_group(
        material_id: MaterialId,
        device: Arc<Device>,
        data: &[u8],
    ) -> (BindGroup, Buffer) {
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

        (
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!(
                    "Material {material_id:?} Uniform Buffer Bind Group"
                )),
                layout: &bind_group_layout,
                entries: &bind_group_entries,
            }),
            uniform_buffer,
        )
    }

    pub fn update_textures(
        &mut self,
        device: Arc<Device>,
        textures: &[&'a Texture],
        replace_indices: &[usize],
    ) {
        let updated_views = (if replace_indices.is_empty() {
            textures
                .iter()
                .map(|tex| (&tex.view, &tex.sampler))
                .collect::<Vec<_>>()
        } else {
            self.views_and_samplers
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    if replace_indices.contains(&i) {
                        (&textures[i].view, &textures[i].sampler)
                    } else {
                        *e
                    }
                })
                .collect::<Vec<_>>()
        });


        self.texture_bind_group = Self::create_texture_bind_group(
            &updated_views,
            device,
        );

        self.views_and_samplers = updated_views;
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

    pub fn uniform_buffer_bind_group(&self) -> Option<&(BindGroup, Buffer)> {
        self.uniform_buffer_and_bind_group.as_ref()
    }
}
