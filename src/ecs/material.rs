#![allow(unused)]
use std::{num::NonZeroU32, rc::Rc, sync::Arc};

use wgpu::{
    util::DeviceExt, BindGroup, BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType,
    Buffer, Device, SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension,
};

use crate::texture::Texture;

pub type MaterialId = (String, String, usize, bool);

#[derive(Debug)]
pub struct Material {
    vertex_shader_path: String,
    fragment_shader_path: String,
    textures: Vec<Rc<Texture>>,
    enabled: bool,
    id: MaterialId,
    texture_bind_group: BindGroup,
    uniform_buffer_and_bind_group: Option<(BindGroup, Buffer)>,
}

impl Material {
    pub fn new(
        vertex_shader_path: &str,
        fragment_shader_path: &str,
        textures: Vec<Rc<Texture>>,
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
            textures,
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

        let views = views_and_samplers
            .iter()
            .map(|(v, _)| *v)
            .collect::<Vec<_>>();
        let samplers = views_and_samplers
            .iter()
            .map(|(_, s)| *s)
            .collect::<Vec<_>>();

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

    pub fn update_textures(&mut self, device: Arc<Device>, textures: &[(Rc<Texture>, usize)]) {
        for (tex, index) in textures {
            self.textures[*index] = tex.clone();
        }

        self.texture_bind_group = Self::create_texture_bind_group(
            &self
                .textures
                .iter()
                .map(|tex| (&tex.view, &tex.sampler))
                .collect::<Vec<_>>(),
            device,
        );
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
