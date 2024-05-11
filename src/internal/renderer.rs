use std::sync::{Arc, RwLock};

use cgmath::Point3;
use wgpu::{util::DeviceExt, BindGroupLayout, RenderPipeline, TextureFormat};

use crate::{engine::{surfacevertex::SurfaceVertex, texture::Texture, texture_loader::{initialize_load_textures, preload_textures}, vertex::{ModelVertex, Vertex}}, gen::{object::RawObject, spotlight::Spotlight}};

use super::{renderpipeline::create_render_pipeline, renderstorage::RenderStorage};

pub struct MainRenderer {
    surface_pipeline: RenderPipeline,
    object_pipeline: RenderPipeline,
    material_bind_group_layout: BindGroupLayout,
    texture_format: wgpu::TextureFormat,
    depth_texture: Texture,
    texture_bindgroup: wgpu::BindGroup,
    texture_bindgroup_layout: wgpu::BindGroupLayout,
    surface_texture_format: wgpu::TextureFormat,
    pub render_storage: RenderStorage,
    shadow_texture: Texture,
    spotlights: Vec<Arc<RwLock<Spotlight>>>
}

impl MainRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, surface_texture_format: wgpu::TextureFormat, camera_bindgroup_layout: &BindGroupLayout, screendims: (u32, u32)) -> Self {
        let texture_format = wgpu::TextureFormat::Rgba8UnormSrgb;
        
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
            surface_texture_format,
            Some(TextureFormat::Depth32Float),
            &[SurfaceVertex::desc()],
            "res/shaders/surfaceshader.wgsl",
            true,
            true
        );

        let object_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("object pipeline layout"),
            bind_group_layouts: &[&texture_bindgroup_layout, &camera_bindgroup_layout],
            push_constant_ranges: &[]
        });

        let object_pipeline = create_render_pipeline(
            "object_pipeline",
            device,
            &object_pipeline_layout,
            surface_texture_format,
            Some(TextureFormat::Depth32Float),
            &[ModelVertex::desc(), RawObject::desc()],
            "res/shaders/objectshader.wgsl",
            true,
            true
        );

        let depth_texture = Texture::from_empty("depth texture", &device, wgpu::TextureFormat::Depth32Float, screendims.0, screendims.1, wgpu::FilterMode::Linear);

        let shadow_texture = Texture::from_empty_shadows("shadow texture", &device, wgpu::TextureFormat::Depth32Float, 512, 512, wgpu::FilterMode::Linear, 100);

        Self {
            material_bind_group_layout,
            surface_pipeline,
            texture_format,
            texture_bindgroup,
            texture_bindgroup_layout,
            depth_texture,
            object_pipeline,
            surface_texture_format,
            render_storage: RenderStorage::new(),
            shadow_texture,
            spotlights: Vec::new()
        }
    }

    pub fn create_spotlight(&mut self, position: Point3<f32>, target: Point3<f32>) -> Arc<RwLock<Spotlight>> {
        let view = self.shadow_texture.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("shadow"),
            format: None,
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: Some(1)
        });

        let spotlight = Spotlight::new(position, target, 70., 1., 200., view);

        let lock = Arc::new(RwLock::new(spotlight));

        self.spotlights.push(lock.clone());

        lock
    }

    pub fn render_objects(&mut self, 
        device: &wgpu::Device, 
        output_texture: &mut wgpu::SurfaceTexture, 
        output_view: &wgpu::TextureView, 
        encoder: &mut wgpu::CommandEncoder,
        camera_bindgroup: &wgpu::BindGroup
    ) {
        let mut buffers: Vec<wgpu::Buffer> = Vec::new();

        let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("shadow render pass"),
            color_attachments: &[],
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
            occlusion_query_set: None
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("object render pass"),
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
        render_pass.set_pipeline(&self.object_pipeline);
        render_pass.set_bind_group(0, &self.texture_bindgroup, &[]);
        render_pass.set_bind_group(1, camera_bindgroup, &[]);

        for obj in self.render_storage.get_objects() {
            let raw = vec![obj.get_raw().clone()];
            
            let obj_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("object buffer"),
                contents: bytemuck::cast_slice(&raw),
                usage: wgpu::BufferUsages::VERTEX
            });

            buffers.push(obj_buffer); //can't have both a mutable and immutable reference at the same time :(
        }

        let mut i = 0;

        for obj in self.render_storage.get_objects() {
            let model = obj.get_model();

            for mesh in &model.meshes {
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, buffers[i].slice(..));
                render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
            }
            i += 1;
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