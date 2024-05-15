use std::{mem, sync::Arc};

use cgmath::{Quaternion, Vector3};

use crate::engine::model::Model;

pub struct Object {
    name: String,
    position: Vector3<f32>,
    scale: Vector3<f32>,
    orientation: Quaternion<f32>,
    model: Arc<Model>,
    raw_object: RawObject
}

impl Object {
    pub fn new(name: String, model: Arc<Model>) -> Self {
        let scale = Vector3::new(1., 1., 1.);
        let position = Vector3::new(0., 0., 0.);
        let orientation = Quaternion::from_sv(1., Vector3::unit_z());

        let model_matrix = cgmath::Matrix4::from_translation(position)
            * cgmath::Matrix4::from(orientation) 
            * cgmath::Matrix4::from_nonuniform_scale(scale.x * 0.5, scale.y * 0.5, scale.z * 0.5);

        Self {
            name,
            model,
            scale,
            position,
            orientation,
            raw_object: RawObject {
                model: model_matrix.into(),
                normal: cgmath::Matrix3::from(orientation).into()
            }
        }
    }

    pub fn get_raw(&self) -> &RawObject {
        &self.raw_object
    }

    pub fn update_raw(&mut self) {
        let model_matrix = cgmath::Matrix4::from_translation(self.position)
            * cgmath::Matrix4::from(self.orientation) 
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.x * 0.5, self.scale.y * 0.5, self.scale.z * 0.5);

        self.raw_object = RawObject {
            model: model_matrix.into(),
            normal: cgmath::Matrix3::from(self.orientation).into()
        }
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
        self.update_raw();
    }
    pub fn get_position(&self) -> Vector3<f32> {self.position}
    pub fn set_scale(&mut self, scale: Vector3<f32>) {
        self.scale = scale;
        self.update_raw();
    }
    pub fn get_scale(&self) -> Vector3<f32> {self.scale}
    pub fn set_orientation(&mut self, orientation: Quaternion<f32>) {
        self.orientation = orientation;
        self.update_raw();
    }
    pub fn get_orientation(&self) -> Quaternion<f32> {self.orientation}

    pub fn set_model(&mut self, model: Arc<Model>) {
        self.model = model;
    }
    pub fn get_model(&self) -> &Arc<Model> {&self.model}
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawObject {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3]
}

impl RawObject {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<RawObject>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 12,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }
}