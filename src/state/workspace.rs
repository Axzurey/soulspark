use cgmath::Point3;

use crate::internal::camera::Camera;

pub struct Workspace {
    pub current_camera: Camera,
}

impl Workspace {
    pub fn new(device: &wgpu::Device, camera_bindgroup_layout: &wgpu::BindGroupLayout, width: u32, height: u32) -> Self {
        Self {
            current_camera: Camera::new(Point3::new(0., 0., 0.), -90.0, -20.0, width as f32 / height as f32, 70., device, camera_bindgroup_layout)
        }
    }
}