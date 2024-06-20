use std::sync::Arc;

use cgmath::Point3;
use winit::window::Window;

use crate::{internal::camera::Camera, util::inputservice::InputService, vox::chunk_manager::ChunkManager};

pub struct Workspace {
    pub current_camera: Camera,
    pub chunk_manager: ChunkManager,
    pub input_service: InputService
}

impl Workspace {
    pub fn new(device: &wgpu::Device, camera_bindgroup_layout: &wgpu::BindGroupLayout, width: u32, height: u32, window: Arc<Window>) -> Self {
        Self {
            current_camera: Camera::new(Point3::new(0., 140., 0.), -90.0, -20.0, width as f32 / height as f32, 70., device, camera_bindgroup_layout),
            chunk_manager: ChunkManager::new(),
            input_service: InputService::new(window)
        }
    }
}