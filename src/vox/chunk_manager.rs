use std::{collections::HashMap, sync::{Arc, RwLock}};

use cgmath::{Vector2, Vector3};
use wgpu::util::DeviceExt;

use crate::{blocks::block::{Block, BlockFace}, engine::surfacevertex::{calculate_tangents_inplace_surfacevertex, SurfaceVertex}};

use super::chunk::{xz_to_index, Chunk};

pub struct ChunkManager {
    chunks: HashMap<u32, Chunk>,
    render_distance: u32,
    seed: u32
}

pub fn get_block_at_absolute(x: i32, y: i32, z: i32, chunks: &HashMap<u32, Chunk>) -> Option<Arc<RwLock<dyn Block + Send + Sync>>> {
    if y < 0 || y > 255 {return None};
    let chunk_x = x.div_euclid(16);
    let chunk_z = z.div_euclid(16);

    let chunk = chunks.get(&xz_to_index(chunk_x, chunk_z))?;

    Some(chunk.get_block_at(x.rem_euclid(16) as u32, y as u32, z.rem_euclid(16) as u32))
}

pub fn push_n(vec: &mut Vec<u32>, start: u32, shifts: [u32; 6]) {
    for i in 0..6 {
        vec.push(start + shifts[i]);
    }
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            render_distance: 5,
            seed: 52352
        }
    }

    pub fn generate_chunks(&mut self, device: &wgpu::Device) {
        for x in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
            for z in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
                let chunk = Chunk::new(Vector2::new(x, z), self.seed);
            }
        }
    }

    pub fn mesh_slice(&mut self, device: &wgpu::Device, chunk: &mut Chunk, y_slice: u32) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let mut vertices: Vec<SurfaceVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let pos = Vector3::new(chunk.position.x * 16, 0, chunk.position.y * 16);
        let rel_abs_x = chunk.position.x * 16;
        let rel_abs_z = chunk.position.y * 16;

        for x in 0..16 {
            for z in 0..16 { 
                for y in y_slice * 16..(y_slice + 1) * 16 {
                    let block_at = chunk.get_block_at(x, y, z);

                    let current = block_at.read().unwrap();

                    if current.has_partial_transparency() || !current.does_mesh() {continue;};

                    let front = get_block_at_absolute((x as i32) + rel_abs_x, y as i32, (z as i32) + rel_abs_z + 1, &self.chunks);
                    let back = get_block_at_absolute((x as i32) + rel_abs_x, y as i32, (z as i32) + rel_abs_z - 1, &self.chunks);
                    let up = get_block_at_absolute((x as i32) + rel_abs_x, (y as i32) + 1, (z as i32) + rel_abs_z, &self.chunks);
                    let down = get_block_at_absolute((x as i32) + rel_abs_x, (y as i32) - 1, (z as i32) + rel_abs_z, &self.chunks);
                    let right = get_block_at_absolute((x as i32) + rel_abs_x + 1, y as i32, (z as i32) + rel_abs_z, &self.chunks);
                    let left = get_block_at_absolute((x as i32) + rel_abs_x - 1, y as i32, (z as i32) + rel_abs_z, &self.chunks);
                    
                    if front.is_none() || (front.is_some() && front.clone().unwrap().read().unwrap().has_partial_transparency( )) {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [0, 1, 2, 1, 3, 2]);

                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Front, 0, current.get_surface_textures(BlockFace::Front)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Front, 1, current.get_surface_textures(BlockFace::Front)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Front, 2, current.get_surface_textures(BlockFace::Front)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Front, 3, current.get_surface_textures(BlockFace::Front)));
                    }

                    if back.is_none() || (back.is_some() && back.clone().unwrap().read().unwrap().has_partial_transparency()) {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [2, 1, 0, 2, 3, 1]);

                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32) as f32], BlockFace::Back, 0, current.get_surface_textures(BlockFace::Back)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32) as f32], BlockFace::Back, 1, current.get_surface_textures(BlockFace::Back)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32) as f32], BlockFace::Back, 2, current.get_surface_textures(BlockFace::Back)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32) as f32], BlockFace::Back, 3, current.get_surface_textures(BlockFace::Back)));
                    }

                    if right.is_none() || (right.is_some() && right.clone().unwrap().read().unwrap().has_partial_transparency()) {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [2, 1, 0, 2, 3, 1]);

                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32) as f32], BlockFace::Right, 0, current.get_surface_textures(BlockFace::Right)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Right, 1, current.get_surface_textures(BlockFace::Right)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32) as f32], BlockFace::Right, 2, current.get_surface_textures(BlockFace::Right)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Right, 3, current.get_surface_textures(BlockFace::Right)));
                    }

                    if left.is_none() || (left.is_some() && left.clone().unwrap().read().unwrap().has_partial_transparency()) {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [0, 1, 2, 1, 3, 2]);

                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32) as f32], BlockFace::Left, 0, current.get_surface_textures(BlockFace::Left)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Left, 1, current.get_surface_textures(BlockFace::Left)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32) as f32], BlockFace::Left, 2, current.get_surface_textures(BlockFace::Left)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Left, 3, current.get_surface_textures(BlockFace::Left)));
                    }

                    if up.is_none() || (up.is_some() && up.clone().unwrap().read().unwrap().has_partial_transparency()) {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [0, 1, 2, 1, 3, 2]);

                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32) as f32], BlockFace::Top, 0, current.get_surface_textures(BlockFace::Top)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Top, 1, current.get_surface_textures(BlockFace::Top)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32) as f32], BlockFace::Top, 2, current.get_surface_textures(BlockFace::Top)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Top, 3, current.get_surface_textures(BlockFace::Top)));
                    }

                    if down.is_none() || (down.is_some() && down.clone().unwrap().read().unwrap().has_partial_transparency()) {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [2, 1, 0, 2, 3, 1]);

                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32) as f32], BlockFace::Bottom, 0, current.get_surface_textures(BlockFace::Bottom)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Bottom, 1, current.get_surface_textures(BlockFace::Bottom)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32) as f32], BlockFace::Bottom, 2, current.get_surface_textures(BlockFace::Bottom)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Bottom, 3, current.get_surface_textures(BlockFace::Bottom)));
                    }

                }
            }
        }

        calculate_tangents_inplace_surfacevertex(&mut vertices, &mut indices);

        let ilen = indices.len() as u32;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Chunk Vertex Buffer")),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Index Buffer")),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer, ilen)
    }

    pub fn mesh_chunk(&mut self, device: &wgpu::Device, index: u32) {
        let chunk = self.chunks.get_mut(&index).unwrap();

        
    }
}