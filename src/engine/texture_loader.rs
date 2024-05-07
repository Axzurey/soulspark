use std::{collections::HashMap, fs::File, io::BufReader, num::NonZeroU32, sync::{Arc, Mutex}};
use once_cell::sync::Lazy;
use serde::Deserialize;
use wgpu::{FilterMode, Sampler, TextureView};

use super::texture::{load_binary_sync, Texture};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureType {
    Diffuse,
    Normal,
    Emissive
}

static TEXTURE_INDICES: Lazy<Mutex<HashMap<String, usize>>> = Lazy::new(|| {
    let m = HashMap::new();
    Mutex::new(m)
});

pub fn get_indices_from_texture(texture: &str) -> usize {
    *TEXTURE_INDICES.lock().unwrap().get(&texture.to_owned()).unwrap()
}

#[derive(Deserialize)]
struct ModelLoadData {
    #[serde(rename(deserialize = "path"))]
    model_path: String,
    alias: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum SerFilterMode {
    Linear,
    Nearest
}

impl Into<FilterMode> for SerFilterMode {
    fn into(self) -> FilterMode {
        match self {
            SerFilterMode::Nearest => FilterMode::Nearest,
            SerFilterMode::Linear => FilterMode::Linear
        }
    }
}

#[derive(Deserialize)]
struct TextureLoadData {
    #[serde(rename(deserialize = "path"))]
    texture_path: String,
    alias: String,
    #[serde(rename(deserialize = "type"))]
    texture_type: TextureType,
    filter: SerFilterMode
}

pub struct LoadedTextureData {
    pub path: String,
    pub alias: String,
    pub texture_type: TextureType,
    pub texture: Arc<Texture>
}

pub static LOADED_TEXTURES: Lazy<Mutex<HashMap<String, LoadedTextureData>>> = Lazy::new(|| {
    let m = HashMap::new();
    Mutex::new(m)
});

pub async fn preload_textures(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat
) {
    let file = File::open("res/data/texture_manifest.json").expect("Unable to load model_manifest file");
    let reader = BufReader::new(file);
    let data: Vec<TextureLoadData> = serde_json::from_reader(reader).expect("Invalid model_manifest data");

    let mut lock = LOADED_TEXTURES.lock().unwrap();

    for definition in data {
        let texture = Arc::new(Texture::from_bytes(&definition.alias, device, queue, format, &load_binary_sync(&definition.texture_path).unwrap(), definition.filter.into()));

        lock.insert(definition.alias.clone(), LoadedTextureData {
            path: definition.texture_path,
            alias: definition.alias,
            texture_type: definition.texture_type,
            texture
        });
    }
}

pub fn initialize_load_textures(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    
    let mut texture_indices = TEXTURE_INDICES.lock().unwrap();

    let mut diffuse_texture_map: Vec<Arc<Texture>> = Vec::new();
    let mut normal_texture_map: Vec<Arc<Texture>> = Vec::new();
    let mut emissive_texture_map: Vec<Arc<Texture>> = Vec::new();

    let diff = Arc::new(Texture::from_color("default-diffuse", device, queue, format, [0, 255, 255, 0], 1, 1, wgpu::FilterMode::Nearest));
    let norm = Arc::new(Texture::from_color("default-normal", device, queue, format, [255 / 3, 255 / 3, 255 / 3, 255], 1, 1, wgpu::FilterMode::Nearest));
    let emi = Arc::new(Texture::from_color("default-emissive", device, queue, format, [0, 0, 0, 0], 1, 1, wgpu::FilterMode::Nearest));

    let mut n_diffuse: u32 = 1;
    let mut n_normal: u32 = 1;
    let mut n_emissive: u32 = 1;

    diffuse_texture_map.push(diff);
    normal_texture_map.push(norm);
    emissive_texture_map.push(emi);

    texture_indices.insert("default-diffuse".to_owned(), 0);
    texture_indices.insert("default-normal".to_owned(), 0);
    texture_indices.insert("default-emissive".to_owned(), 0);

    let loaded_textures = LOADED_TEXTURES.lock().unwrap();

    for name in loaded_textures.keys() {
        let texturedata = &loaded_textures[name];

        let alias = texturedata.alias.clone();
    
        match texturedata.texture_type {
            TextureType::Diffuse => {
                diffuse_texture_map.push(texturedata.texture.clone());
                texture_indices.insert(alias.clone(), n_diffuse as usize);
                n_diffuse += 1;
            },
            TextureType::Normal => {
                normal_texture_map.push(texturedata.texture.clone());
                texture_indices.insert(alias.clone(), n_normal as usize);
                n_normal += 1;
            },
            TextureType::Emissive => {
                emissive_texture_map.push(texturedata.texture.clone());
                texture_indices.insert(alias.clone(), n_emissive as usize);
                n_emissive += 1;
            },
        }
    }

    let surface_texture_bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(n_diffuse),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: NonZeroU32::new(n_diffuse),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(n_normal),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: NonZeroU32::new(n_normal),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(n_emissive),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: NonZeroU32::new(n_emissive),
                },
            ],
            label: Some("Surface texture bind group layout")
        }
    );

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Surface texture bind group"),
        layout: &surface_texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureViewArray(
                    &diffuse_texture_map.iter().map(|v| &v.view).collect::<Vec<&TextureView>>()
                )
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::SamplerArray(
                    &diffuse_texture_map.iter().map(|v| &v.sampler).collect::<Vec<&Sampler>>()
                )
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureViewArray(
                    &normal_texture_map.iter().map(|v| &v.view).collect::<Vec<&TextureView>>()
                )
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::SamplerArray(
                    &normal_texture_map.iter().map(|v| &v.sampler).collect::<Vec<&Sampler>>()
                )
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureViewArray(
                    &emissive_texture_map.iter().map(|v| &v.view).collect::<Vec<&TextureView>>()
                )
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::SamplerArray(
                    &emissive_texture_map.iter().map(|v| &v.sampler).collect::<Vec<&Sampler>>()
                )
            }
        ]
    });

    (texture_bind_group, surface_texture_bind_group_layout)
}