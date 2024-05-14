use std::sync::Arc;

use cgmath::{ortho, perspective, InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation, Vector3};
use glam::Vec3;

use crate::{engine::{color4::Color4, texture::Texture}, internal::camera::{OPENGL_TO_WGPU_MATRIX, OPENGL_TO_WGPU_MATRIX_GLAM}};

#[derive(getset::Getters, getset::MutGetters)]
pub struct Spotlight {
    #[getset(get)]
    position: Point3<f32>,
    target_position: Point3<f32>,
    near: f32,
    far: f32,
    fov: f32,
    #[getset(skip)]
    raw_spotlight: RawSpotLight,
    #[getset(skip)]
    pub texture_view: wgpu::TextureView,
    color: Color4
}

impl Spotlight {
    pub fn new(position: Point3<f32>, target_position: Point3<f32>, fov: f32, near: f32, far: f32, aspect: f32, texture_view: wgpu::TextureView) -> Self {
        let proj = perspective(Rad(fov.to_radians()), aspect, near, far);

        let view = Matrix4::look_to_rh(
            position,
            (Point3::new(0., 0., 0.) - position).normalize().into(),
            Vector3::unit_y()
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
                model: view_proj.into(),
                color: [0., 1., 1., 1.]
            },
            texture_view,
            color: Color4::new(1., 0., 0., 1.)
        }
    }

    pub fn update<F>(&mut self, mut callback: F) where F: FnMut(&mut Spotlight) {
        callback(self);
        self.update_raw();
    }

    pub fn get_raw(&self) -> &RawSpotLight {
        &self.raw_spotlight
    }
    fn update_raw(&mut self) {
        let proj = OPENGL_TO_WGPU_MATRIX_GLAM * glam::Mat4::perspective_rh(self.fov.to_radians(), 1., self.near, self.far);

        let view = glam::Mat4::look_at_rh(
            Vec3::new(self.position.x, self.position.y, self.position.z), 
            Vec3::new(self.target_position.x, self.target_position.y, self.target_position.z),
            Vec3::Z
        );

        let view_proj = proj * view;

        self.raw_spotlight.position = [self.position.x, self.position.y, self.position.z, 1.];
        self.raw_spotlight.model = view_proj.to_cols_array_2d();
        self.raw_spotlight.color = self.color.into();
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawSpotLight {
    pub position: [f32; 4],
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4]
}