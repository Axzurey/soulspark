use crate::engine::texture::Texture;
use std::sync::Arc;

pub struct Material {
    name: String,
    diffuse_texture: Arc<Texture>,
    normal_texture: Arc<Texture>,
    emissive_texture: Arc<Texture>,
    bind_group: wgpu::BindGroup
}