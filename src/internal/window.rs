use std::sync::Arc;
use winit::{event_loop::EventLoop, window::Window, window::WindowBuilder};

use crate::state::workspace::Workspace;

pub struct GameWindow<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    window_size: winit::dpi::PhysicalSize<u32>,
    window: Arc<Window>,
    surface_format: wgpu::TextureFormat
}

impl<'a> GameWindow<'a> {
    pub async fn new() -> Self {
        env_logger::init();

        let event_loop = EventLoop::new().unwrap();

        let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

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
             | wgpu::Features::BGRA8UNORM_STORAGE,
            required_limits: wgpu::Limits {
                max_sampled_textures_per_shader_stage: 121,
                max_samplers_per_shader_stage: 121,
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

        surface.configure(&device, &surface_config);

        Self {
            surface,
            queue,
            device,
            window,
            window_size,
            surface_config,
            surface_format
        }
    }

    fn on_next_frame(workspace: &mut Workspace) {

        workspace.current_camera.update_matrices();


    }
}