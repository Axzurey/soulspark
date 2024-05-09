use super::vertex::ModelVertex;

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub face_indices: Vec<u32>,
    pub vertices: Vec<ModelVertex>
}