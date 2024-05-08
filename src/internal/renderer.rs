use wgpu::{BindGroupLayout, RenderPipeline};

use crate::engine::{surfacevertex::SurfaceVertex, texture::Texture, texture_loader::{initialize_load_textures, preload_textures}, vertex::Vertex};

use super::renderpipeline::create_render_pipeline;

pub struct MainRenderer {
    surface_pipeline: RenderPipeline,
    //object_pipeline: RenderPipeline,
    material_bind_group_layout: BindGroupLayout,
    texture_format: wgpu::TextureFormat,
    depth_texture: Texture,
    texture_bindgroup: wgpu::BindGroup,
    texture_bindgroup_layout: wgpu::BindGroupLayout
}

impl MainRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, texture_format: wgpu::TextureFormat, camera_bindgroup_layout: &BindGroupLayout, screendims: (u32, u32)) -> Self {
        preload_textures(device, queue, texture_format);

        let (texture_bindgroup, texture_bindgroup_layout) = initialize_load_textures(device, queue, texture_format);

        let material_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Main renderer material bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable:  true }
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable:  true }
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ]
        });

        let surface_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("surface pipeline layout"),
            bind_group_layouts: &[&texture_bindgroup_layout, &camera_bindgroup_layout],
            push_constant_ranges: &[]
        });

        let surface_pipeline = create_render_pipeline(
            "surface_pipeline",
            device,
            &surface_pipeline_layout,
            texture_format,
            None,
            &[SurfaceVertex::desc()],
            "../shaders/surfaceshader.wgsl",
            true,
            false
        );

        let depth_texture = Texture::from_color("depth texture", &device, &queue, texture_format, [0, 0, 0 ,0], screendims.0, screendims.1, wgpu::FilterMode::Linear);

        Self {
            material_bind_group_layout,
            surface_pipeline,
            texture_format,
            texture_bindgroup,
            texture_bindgroup_layout,
            depth_texture
        }
    }

    pub fn render_surface(&mut self, 
        device: &wgpu::Device, 
        output_texture: &mut wgpu::Texture, 
        output_view: &wgpu::TextureView, 
        encoder: &mut wgpu::CommandEncoder,
        camera_bindgroup: &wgpu::BindGroup
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("surface render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                view: &output_view, 
                resolve_target: None, 
                ops: wgpu::Operations { 
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0
                    }),
                    store: wgpu::StoreOp::Store
                }
            })],
            depth_stencil_attachment: Some(
                wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(
                        wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store
                        }
                    ),
                    stencil_ops: None,
                }
            ),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&self.surface_pipeline);
    }
}