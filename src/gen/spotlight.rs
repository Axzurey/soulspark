use cgmath::{ortho, Matrix4, Point3, Quaternion, Rotation, Vector3};
use glam::Vec3;

use crate::{engine::texture::Texture, internal::camera::{OPENGL_TO_WGPU_MATRIX, OPENGL_TO_WGPU_MATRIX_GLAM}};

pub struct Spotlight {
    position: Point3<f32>,
    target_position: Point3<f32>,
    near: f32,
    far: f32,
    fov: f32,
    raw_spotlight: RawSpotLight,
    texture_view: wgpu::TextureView
}

impl Spotlight {
    pub fn new(position: Point3<f32>, target_position: Point3<f32>, fov: f32, near: f32, far: f32, texture_view: wgpu::TextureView) -> Self {
        let proj = OPENGL_TO_WGPU_MATRIX_GLAM * glam::Mat4::perspective_rh(fov.to_radians(), 1., near, far);

        let view = glam::Mat4::look_at_rh(
            Vec3::new(position.x, position.y, position.z), 
            Vec3::new(target_position.x, target_position.y, target_position.z),
            Vec3::Z
        );

        let view_proj = proj * view;

        Self {
            position,
            target_position,
            near,
            far,
            fov,
            raw_spotlight: RawSpotLight {
                position: [position.x, position.y, position.z, 1.],
                model: view_proj.to_cols_array_2d()
            },
            texture_view
        }
    }
    pub fn update_raw(&mut self) {
        let proj = OPENGL_TO_WGPU_MATRIX_GLAM * glam::Mat4::perspective_rh(self.fov.to_radians(), 1., self.near, self.far);

        let view = glam::Mat4::look_at_rh(
            Vec3::new(self.position.x, self.position.y, self.position.z), 
            Vec3::new(self.target_position.x, self.target_position.y, self.target_position.z),
            Vec3::Z
        );

        let view_proj = proj * view;

        self.raw_spotlight.position = [self.position.x, self.position.y, self.position.z, 1.];
        self.raw_spotlight.model = view_proj.to_cols_array_2d();
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawSpotLight {
    position: [f32; 4],
    model: [[f32; 4]; 4]
}