use std::{mem, ops::BitOrAssign};

use cgmath::InnerSpace;
use wgpu::vertex_attr_array;

use crate::blocks::block::BlockFace;

use super::vertex::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SurfaceVertex {
    pub d0: u32,
    pub illumination: u32
}

impl SurfaceVertex {
    pub fn from_position(pos: [u32; 3], face: BlockFace, nth: u32, texture_indices: (usize, usize, usize), illumination: u32) -> SurfaceVertex {
        let face_dir = match face {
            BlockFace::Top => 0,
            BlockFace::Bottom => 1,
            BlockFace::Right => 2,
            BlockFace::Left => 3,
            BlockFace::Front => 4,
            BlockFace::Back => 5,
        };
        // 15 bits for pos
        // 3 bits for direction
        // 2 bits for normal
        // 6 bits for diffuse texture index

        let mut d0 = 0;
        // let mut d1 = 0;

        d0.bitor_assign(pos[0]);
        d0.bitor_assign(pos[1] << 5);
        d0.bitor_assign(pos[2] << 10);
        d0.bitor_assign(face_dir << 15);
        d0.bitor_assign(nth << 18);
        d0.bitor_assign((texture_indices.0 as u32) << 20);

        // d1.bitor_assign(texture_indices.0 as u32);
        // d1.bitor_assign((texture_indices.1 as u32) << 8);
        // d1.bitor_assign((texture_indices.2 as u32) << 16);
    
        SurfaceVertex {
            d0, illumination
        }
    }
}

impl Vertex for SurfaceVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SurfaceVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[u32; 1]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Uint32,
                },
            ]
        }
    }
}