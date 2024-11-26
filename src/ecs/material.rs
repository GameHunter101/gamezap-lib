use wgpu::{BindGroup, Buffer};

use crate::engine_support::texture_support::Texture;

pub type MaterialId = u32;

pub struct Material {
    id: MaterialId,
    textures: Vec<Texture>,
    bind_groups: Vec<BindGroup>,
    buffers: Vec<Buffer>,
}
