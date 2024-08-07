use std::{mem::{self, size_of}, num::NonZeroU32, sync::{Arc, RwLock, RwLockReadGuard}};

use cgmath::{Matrix3, Matrix4, MetricSpace, Point3, SquareMatrix, Vector3};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use stopwatch::Stopwatch;
use wgpu::{util::DeviceExt, BindGroupLayout, RenderPipeline, TextureFormat};

use crate::{engine::{surfacevertex::SurfaceVertex, texture::Texture, texture_loader::{initialize_load_textures, preload_textures}, vertex::{ModelVertex, Vertex}}, gen::{object::RawObject, spotlight::{RawSpotLight, Spotlight}}, state::workspace::Workspace, vox::chunk::{local_xz_to_index_dynamic, xz_to_index, Chunk, ChunkDataVertex, ChunkState}};

use super::{camera::Camera, depthsort::sort_chunk_transparent_quads, renderpipeline::create_render_pipeline, renderstorage::RenderStorage};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct RendererGlobals {
    current_light_model: [[f32; 4]; 4]
}

pub struct MainRenderer {
    surface_pipeline: RenderPipeline,
    surface_pipeline_wireframe: RenderPipeline,
    transparent_surface_pipeline: RenderPipeline,
    object_pipeline: RenderPipeline,
    material_bind_group_layout: BindGroupLayout,
    texture_format: wgpu::TextureFormat,
    depth_texture: Texture,
    texture_bindgroup: wgpu::BindGroup,
    texture_bindgroup_layout: wgpu::BindGroupLayout,
    surface_texture_format: wgpu::TextureFormat,
    pub render_storage: RenderStorage,
    shadow_texture: Texture,
    spotlights: Vec<Arc<RwLock<Spotlight>>>,
    shadow_pipeline: wgpu::RenderPipeline,
    shadow_bindgroup_layout: wgpu::BindGroupLayout,
    globals: RendererGlobals,
    global_bindgroup_layout: wgpu::BindGroupLayout,
    shadow_prebindgroup_layout: wgpu::BindGroupLayout,
    width: u32,
    height: u32,
    wireframe_mode: bool
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
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

        let shadow_bindgroup_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadows bindgroup"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ]
        });

        let global_bindgroup_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("global bindgroup layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });

        let surface_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("surface pipeline layout"),
            bind_group_layouts: &[&texture_bindgroup_layout, &camera_bindgroup_layout],
            push_constant_ranges: &[]
        });

        let shadow_prebindgroup_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("empty shadow bindgroup layout"),
            entries: &[]
        });

        let surface_pipeline = create_render_pipeline(
            "surface_pipeline",
            device,
            &surface_pipeline_layout,
            surface_texture_format,
            Some(TextureFormat::Depth32Float),
            &[SurfaceVertex::desc(), ChunkDataVertex::desc()],
            "res/shaders/surfaceshader.wgsl",
            true,
            true,
            None,
            false,
            false,
            false,
            false
        );

        let surface_pipeline_wireframe = create_render_pipeline(
            "surface_pipeline wireframe",
            device,
            &surface_pipeline_layout,
            surface_texture_format,
            Some(TextureFormat::Depth32Float),
            &[SurfaceVertex::desc(), ChunkDataVertex::desc()],
            "res/shaders/surfaceshader.wgsl",
            true,
            true,
            None,
            false,
            false,
            false,
            true
        );

        let transparent_surface_pipeline = create_render_pipeline(
            "surface_pipeline",
            device,
            &surface_pipeline_layout,
            surface_texture_format,
            Some(TextureFormat::Depth32Float),
            &[SurfaceVertex::desc(), ChunkDataVertex::desc()],
            "res/shaders/surfaceshader.wgsl",
            true,
            false,
            None,
            false,
            false,
            false,
            false
        );

        let object_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("object pipeline layout"),
            bind_group_layouts: &[&texture_bindgroup_layout, &camera_bindgroup_layout, &global_bindgroup_layout, &shadow_bindgroup_layout],
            push_constant_ranges: &[]
        });

        let shadow_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("object pipeline layout"),
            bind_group_layouts: &[&texture_bindgroup_layout, &camera_bindgroup_layout, &global_bindgroup_layout, &shadow_prebindgroup_layout],
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
            true,
            None,
            false,
            false,
            false,
            false
        );

        let shadow_pipeline = create_render_pipeline(
            "shadow pipeline",
            device,
            &shadow_pipeline_layout,
            surface_texture_format,
            Some(TextureFormat::Depth32Float),
            &[ModelVertex::desc(), RawObject::desc()],
            "res/shaders/objectshader.wgsl",
            true,
            true,
            Some(wgpu::DepthBiasState {
                constant: 2, // corresponds to bilinear filtering
                slope_scale: 2.0,
                clamp: 0.0,
            }),
            true,
            true,
            true,
            false
        );

        let depth_texture = Texture::from_empty("depth texture", &device, wgpu::TextureFormat::Depth32Float, screendims.0, screendims.1, wgpu::FilterMode::Linear);

        let shadow_texture = Texture::from_empty_shadows("shadow texture", &device, wgpu::TextureFormat::Depth32Float, 512, 512, wgpu::FilterMode::Linear, 100);

        let globals = RendererGlobals {
            current_light_model: Matrix4::identity().into()
        };

        Self {
            material_bind_group_layout,
            surface_pipeline,
            transparent_surface_pipeline,
            texture_format,
            texture_bindgroup,
            texture_bindgroup_layout,
            depth_texture,
            object_pipeline,
            surface_texture_format,
            render_storage: RenderStorage::new(),
            shadow_texture,
            spotlights: Vec::new(),
            shadow_pipeline,
            shadow_bindgroup_layout,
            globals,
            global_bindgroup_layout,
            shadow_prebindgroup_layout,
            width: screendims.0,
            height: screendims.1,
            wireframe_mode: false,
            surface_pipeline_wireframe
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
            base_array_layer: 0, //change this for each one
            array_layer_count: Some(1)
        });

        let spotlight = Spotlight::new(position, target, 70., 0.1, 400., self.width as f32 / self.height as f32, view);

        let lock = Arc::new(RwLock::new(spotlight));

        self.spotlights.push(lock.clone());

        lock//view, forwarddepth
    }

    pub fn render_objects(&mut self, 
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_texture: &mut wgpu::SurfaceTexture, 
        output_view: &wgpu::TextureView, 
        encoder: &mut wgpu::CommandEncoder,
        camera_bindgroup: &wgpu::BindGroup,
        workspace: &Workspace
    ) {
        // let frame_start = Stopwatch::start_new();
        // let spotlight_raws: Vec<RawSpotLight> = self.spotlights.par_iter().map(|v| v.read().unwrap().get_raw().clone()).collect();

        // let spotlight_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("object buffer"),
        //     contents: bytemuck::cast_slice(&spotlight_raws),
        //     usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC
        // });

        // let empty_shadows_bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("empty shadow bindgroup"),
        //     layout: &self.shadow_prebindgroup_layout,
        //     entries: &[]
        // });

        // let shadows_bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("shadow bindgroup"),
        //     layout: &self.shadow_bindgroup_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: spotlight_buffer.as_entire_binding()
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::TextureView(&self.shadow_texture.view),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 2,
        //             resource: wgpu::BindingResource::Sampler(&self.shadow_texture.sampler),
        //         },
        //     ]
        // });

        // let mut buffers: Vec<wgpu::Buffer> = Vec::new();

        // for obj in self.render_storage.get_objects() {
        //     let raw = vec![obj.get_raw().clone()];
            
        //     let obj_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //         label: Some("some object buffer"),
        //         contents: bytemuck::cast_slice(&raw),
        //         usage: wgpu::BufferUsages::VERTEX
        //     });

        //     buffers.push(obj_buffer); //can't have both a mutable and immutable reference at the same time :(
        // }
        // let mut index = 0;

        // let default_object_buffer = {
        //     let raw = vec![RawObject {
        //         model: Matrix4::identity().into(),
        //         normal: Matrix3::identity().into()
        //     }];

        //     device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //         label: Some("default object buffer"),
        //         contents: bytemuck::cast_slice(&raw),
        //         usage: wgpu::BufferUsages::VERTEX
        //     })
        // };

        // for light in &self.spotlights {

        //     let read = light.read().unwrap();

        //     self.globals.current_light_model = read.get_raw().model;

        //     let global_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //         label: Some("Camera Buffer"),
        //         contents: bytemuck::cast_slice(&[self.globals]),
        //         usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        //     });
    
        //     let global_bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //         layout: &self.global_bindgroup_layout,
        //         entries: &[
        //             wgpu::BindGroupEntry {
        //                 binding: 0,
        //                 resource: global_buffer.as_entire_binding()
        //             },
        //         ],
        //         label: Some("Camera bind group :)")
        //     });

        //     let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //         label: Some("shadow render pass"),
        //         color_attachments: &[],
        //         depth_stencil_attachment: Some(
        //             wgpu::RenderPassDepthStencilAttachment {
        //                 view: &read.texture_view,
        //                 depth_ops: Some(
        //                     wgpu::Operations {
        //                         load: wgpu::LoadOp::Clear(1.0),
        //                         store: wgpu::StoreOp::Store
        //                     }
        //                 ),
        //                 stencil_ops: None,
        //             }
        //         ),
        //         timestamp_writes: None,
        //         occlusion_query_set: None
        //     });
    
        //     shadow_pass.set_pipeline(&self.shadow_pipeline);
        //     shadow_pass.set_bind_group(0, &self.texture_bindgroup, &[]);
        //     shadow_pass.set_bind_group(1, camera_bindgroup, &[]);
        //     shadow_pass.set_bind_group(2, &global_bindgroup, &[]);
        //     shadow_pass.set_bind_group(3, &empty_shadows_bindgroup, &[]);

        //     let mut i = 0;

        //     for obj in self.render_storage.get_objects() {
        //         let model = obj.get_model();
    
        //         for mesh in &model.meshes {
        //             shadow_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        //             shadow_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        //             shadow_pass.set_vertex_buffer(1, buffers[i].slice(..));
        //             shadow_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        //         }
        //         i += 1;
        //     }

        //     for (index, chunk) in workspace.chunk_manager.chunks.iter() {
        //         let out = chunk.get_solid_buffers();

        //         for (vertex_buffer, index_buffer, ilen) in out {
        //             if *ilen == 0 {continue};
        //             shadow_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        //             shadow_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        //             shadow_pass.set_vertex_buffer(1, default_object_buffer.slice(..));
        //             shadow_pass.draw_indexed(0..*ilen, 0, 0..1);
        //         }
        //     }
    
        //     drop(shadow_pass);
        //     index += 1;
        // }

        // let global_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Camera Buffer"),
        //     contents: bytemuck::cast_slice(&[self.globals]),
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        // });

        // let global_bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &self.global_bindgroup_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: global_buffer.as_entire_binding()
        //         },
        //     ],
        //     label: Some("Camera bind group :)")
        // });

        // let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //     label: Some("object render pass"),
        //     color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
        //         view: &output_view, 
        //         resolve_target: None, 
        //         ops: wgpu::Operations { 
        //             load: wgpu::LoadOp::Clear(wgpu::Color {
        //                 r: 0.1,
        //                 g: 0.2,
        //                 b: 0.3,
        //                 a: 1.0
        //             }),
        //             store: wgpu::StoreOp::Store
        //         }
        //     })],
        //     depth_stencil_attachment: Some(
        //         wgpu::RenderPassDepthStencilAttachment {
        //             view: &self.depth_texture.view,
        //             depth_ops: Some(
        //                 wgpu::Operations {
        //                     load: wgpu::LoadOp::Clear(1.0),
        //                     store: wgpu::StoreOp::Discard
        //                 }
        //             ),
        //             stencil_ops: None,
        //         }
        //     ),
        //     timestamp_writes: None,
        //     occlusion_query_set: None,
        // });
        // render_pass.set_pipeline(&self.object_pipeline);
        // render_pass.set_bind_group(0, &self.texture_bindgroup, &[]);
        // render_pass.set_bind_group(1, camera_bindgroup, &[]);
        // render_pass.set_bind_group(2, &global_bindgroup, &[]);
        // render_pass.set_bind_group(3, &shadows_bindgroup, &[]);

        // let mut i = 0;

        // for obj in self.render_storage.get_objects() {
        //     let model = obj.get_model();

        //     for mesh in &model.meshes {
        //         render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        //         render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        //         render_pass.set_vertex_buffer(1, buffers[i].slice(..));
        //         render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        //     }
        //     i += 1;
        // }

        // for (_index, chunk) in workspace.chunk_manager.chunks.iter() {
        //     let out = chunk.get_solid_buffers();

        //     for (vertex_buffer, index_buffer, ilen) in out {
        //         if *ilen == 0 {continue};
        //         render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        //         render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        //         render_pass.set_vertex_buffer(1, default_object_buffer.slice(..));
        //         render_pass.draw_indexed(0..*ilen, 0, 0..1);
        //     }
        // }
        // println!("Frame took {}ms", frame_start.elapsed_ms());
    }

    pub fn render_surface(&mut self, 
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_texture: &mut wgpu::SurfaceTexture, 
        output_view: &wgpu::TextureView, 
        encoder: &mut wgpu::CommandEncoder,
        workspace: &mut Workspace
    ) {
        if (workspace.chunk_manager.chunks.len() as u32) < (workspace.chunk_manager.render_distance * 2 + 1).pow(2) {return}
        let t = Stopwatch::start_new();
        let camera_bindgroup = &workspace.current_camera.bindgroup;

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
        render_pass.set_pipeline(if self.wireframe_mode {&self.surface_pipeline_wireframe} else {&self.surface_pipeline});
        render_pass.set_bind_group(0, &self.texture_bindgroup, &[]);
        render_pass.set_bind_group(1, camera_bindgroup, &[]);


        let mut outeri = 0;
        for (index, chunk) in workspace.chunk_manager.chunk_buffers.iter() {
            let chunkref = workspace.chunk_manager.chunks.get(index).unwrap();
            let out = chunk.get_solid_buffers();

            let mut i = 0;

            for t in out {
                if chunkref.states[i] != ChunkState::Ready {i += 1; continue};
                let (vertex_buffer, index_buffer, ilen) = t.as_ref().unwrap();
                if *ilen == 0 {
                    i += 1;
                    continue;
                };
                
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, chunk.slice_vertex_buffers[i].slice(..));
                render_pass.draw_indexed(0..*ilen, 0, 0..1);

                i += 1;
            }
            outeri += 1;
        }
        drop(render_pass);

        let mut transparency_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("transparent render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                view: &output_view, 
                resolve_target: None, 
                ops: wgpu::Operations { 
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store
                }
            })],
            depth_stencil_attachment: Some(
                wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: None,
                    stencil_ops: None,
                }
            ),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        transparency_render_pass.set_pipeline(&self.transparent_surface_pipeline);
        transparency_render_pass.set_bind_group(0, &self.texture_bindgroup, &[]);
        transparency_render_pass.set_bind_group(1, camera_bindgroup, &[]);

        let mut chunks_sorted = workspace.chunk_manager.chunks.clone().iter().flat_map(|v| (0..16).map(|slice| {
            let pos = v.1.position;
            Vector3::new(pos.x as f32, slice as f32, pos.y as f32)
        })).collect::<Vec<_>>();

        let camera_pos = Vector3::new(
            workspace.current_camera.position.x,
            workspace.current_camera.position.y,
            workspace.current_camera.position.z
        );

        chunks_sorted.sort_by(|a, b| {
            let dista = a.distance(camera_pos);
            let distb = b.distance(camera_pos);
    
            dista.partial_cmp(&distb).unwrap()
        });

        // for chunkp in chunks_sorted {
        //     let chunk = workspace.chunk_manager.chunks.get_mut(&xz_to_index(chunkp.x as i32, chunkp.z as i32)).unwrap();
        //     let chunkbuff = workspace.chunk_manager.chunk_buffers.get(&xz_to_index(chunkp.x as i32, chunkp.z as i32)).unwrap();

        //     let buffs = sort_chunk_transparent_quads(device, &workspace.current_camera, chunk, chunkp.y as usize);

        //     match buffs {
        //         None => {},
        //         Some(transparent_tris) => {
        //             transparency_render_pass.set_index_buffer(transparent_tris.1.slice(..), wgpu::IndexFormat::Uint32);
        //             transparency_render_pass.set_vertex_buffer(0, transparent_tris.0.slice(..));
        //             transparency_render_pass.set_vertex_buffer(1, chunkbuff.slice_vertex_buffers[chunkp.y as usize].slice(..));
        //             transparency_render_pass.draw_indexed(0..transparent_tris.2 as u32, 0, 0..1);
        //         }
        //     }
        // }

        // for (_index, chunk) in workspace.chunk_manager.chunks.iter() {
        //     let read = chunk.read();
        //     locks.push(read);
        // }
        
        // let mut outeri = 0;
        // for (_index, chunk) in workspace.chunk_manager.chunks.iter() {
        //     let read = &locks[outeri];

        //     let out = read.get_transparent_buffers();

        //     let mut i = 0;

        //     for t in out {
        //         if read.states[i] != ChunkState::Ready {i += 1; continue};
        //         let (vertex_buffer, index_buffer, ilen) = t.as_ref().unwrap();
        //         if *ilen == 0 {
        //             i += 1;
        //             continue;
        //         };
                
        //         transparency_render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        //         transparency_render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        //         transparency_render_pass.set_vertex_buffer(1, read.slice_vertex_buffers[i].slice(..));
        //         transparency_render_pass.draw_indexed(0..*ilen, 0, 0..1);

        //         i += 1;
        //     }
        //     outeri += 1;
        // }
        drop(transparency_render_pass);
        //println!("frame render: {}ms", t.elapsed_ms());
    }
}