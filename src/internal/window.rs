use std::sync::{Arc, RwLock};
use egui_wgpu::ScreenDescriptor;
use instant::Duration;
use winit::{event_loop::EventLoop, window::Window, window::WindowBuilder};

use crate::{gui::{elements::screenui::ScreenUi, guirenderer::GuiRenderer}, internal::renderer::MainRenderer, state::workspace::Workspace};

pub struct GameWindow<'a> {
    surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    pub window_size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,
    surface_format: wgpu::TextureFormat,
    pub renderer: MainRenderer,
    pub gui_renderer: GuiRenderer,
    pub camera_bindgroup_layout: wgpu::BindGroupLayout,
    pub screenui: Arc<RwLock<ScreenUi>>
}

impl<'a> GameWindow<'a> {
    pub async fn new(window: Arc<Window>) -> Self {
        env_logger::init();

        let window_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            flags: wgpu::InstanceFlags::empty(),
            backends: wgpu::Backends::DX12,
            dx12_shader_compiler: Default::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false
            }
        ).await.unwrap();

        println!("ADAPTER: {:?}", adapter.get_info());

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::TEXTURE_BINDING_ARRAY
             | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING 
             | wgpu::Features::BGRA8UNORM_STORAGE | wgpu::Features::DEPTH_CLIP_CONTROL,
            required_limits: wgpu::Limits {
                max_sampled_textures_per_shader_stage: 121,
                max_samplers_per_shader_stage: 121,
                max_bind_groups: 5,
                ..Default::default()
            },
            label: None,
        }, None).await.unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        println!("{:?}", surface_capabilities.formats);

        let surface_format = surface_capabilities.formats.iter()
            .copied().find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[surface_capabilities.formats.len() - 1]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        let camera_bindgroup_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            ],
            label: Some("camera bind group layout :)"),
        });

        let renderer = MainRenderer::new(
            &device, &queue, surface_format, 
            &camera_bindgroup_layout, (window_size.width, window_size.height)
        );

        surface.configure(&device, &surface_config);

        let gui_renderer = GuiRenderer::new(&device, surface_format, None, 1, &window);

        let screenui = ScreenUi::new("screen-root".to_string());

        Self {
            surface,
            queue,
            device,
            window,
            window_size,
            surface_config,
            surface_format,
            renderer,
            camera_bindgroup_layout,
            gui_renderer,
            screenui
        }
    }

    pub fn on_next_frame(&mut self, workspace: &mut Workspace, dt: f32) {

        workspace.current_camera.update_camera(dt);
        workspace.current_camera.update_matrices(&self.queue);

        let mut output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Primary Encoder")
        });

        self.renderer.render_surface(&self.device, &self.queue, &mut output, &view, &mut encoder, &workspace.current_camera.bindgroup, &workspace);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.window_size.width, self.window_size.height],
            pixels_per_point: self.window.scale_factor() as f32
        };

        self.gui_renderer.draw(&self.device, &self.queue, &mut encoder, &self.window, &view, screen_descriptor, |ctx| {
            self.screenui.write().unwrap().render(ctx)
        });

        self.queue.submit(std::iter::once(encoder.finish()));
        
        output.present();
    }
}