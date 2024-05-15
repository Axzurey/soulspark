use std::mem;

use cgmath::InnerSpace;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
    pub diffuse_texture_index: u32,
    pub normal_texture_index: u32,
    pub emissive_texture_index: u32
}

pub fn calculate_tangents_inplace_modelvertex(vertices: &mut Vec<ModelVertex>, indices: &mut Vec<u32>) {

    let mut triangles_incl = vec![0; vertices.len()];

    for c in indices.chunks(3) {
        let v0 = vertices.get(c[0] as usize).unwrap();
        let v1 = vertices.get(c[1] as usize).unwrap();
        let v2 = vertices.get(c[2] as usize).unwrap();

        let pos0: cgmath::Vector3<_> = v0.position.into();
        let pos1: cgmath::Vector3<_> = v1.position.into();
        let pos2: cgmath::Vector3<_> = v2.position.into();

        let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
        let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
        let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

        let delta_pos1 = pos1 - pos0;
        let delta_pos2 = pos2 - pos0;

        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;

        let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

        vertices[c[0] as usize].tangent =
            (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
        vertices[c[1] as usize].tangent =
            (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
        vertices[c[2] as usize].tangent =
            (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
        vertices[c[0] as usize].bitangent = 
            (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
        vertices[c[1] as usize].bitangent =
            (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
        vertices[c[2] as usize].bitangent =
            (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();

        triangles_incl[c[0] as usize] += 1;
        triangles_incl[c[1] as usize] += 1;
        triangles_incl[c[2] as usize] += 1;
    }

    for (i, n) in triangles_incl.into_iter().enumerate() {
        let denom = 1.0 / n as f32;
        let v = &mut vertices[i];
        //todo: double check if these are supposed to be normalized
        v.tangent = (cgmath::Vector3::from(v.tangent) * denom).normalize().into();
        v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom).normalize().into();
    }
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 14]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}