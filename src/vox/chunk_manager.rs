use std::{collections::HashMap, sync::{Arc, RwLock}};

use cgmath::{Vector2, Vector3};
use noise::Perlin;
use rand::{RngCore, SeedableRng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use stopwatch::Stopwatch;
use wgpu::util::DeviceExt;

use crate::{blocks::block::{Block, BlockFace, BlockType}, engine::surfacevertex::{calculate_tangents_inplace_surfacevertex, SurfaceVertex}};

use super::chunk::{xz_to_index, Chunk};

pub struct ChunkManager {
    pub chunks: HashMap<u32, Chunk>,
    render_distance: u32,
    seed: u32,
    noise_gen: Perlin
}

pub fn get_block_at_absolute(x: i32, y: i32, z: i32, chunks: &HashMap<u32, Chunk>) -> Option<&BlockType> {
    if y < 0 || y > 255 {return None};
    let chunk_x = x.div_euclid(16);
    let chunk_z = z.div_euclid(16);

    let chunk = chunks.get(&xz_to_index(chunk_x, chunk_z))?;

    Some(chunk.get_block_at(x as u32, y as u32, z as u32))
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
            render_distance: 20,
            seed: 52352,
            noise_gen: Perlin::new(rand::rngs::StdRng::seed_from_u64(52352).next_u32())
        }
    }

    pub fn generate_chunks(&mut self) {

        for x in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
            for z in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
                let chunk = Chunk::new(Vector2::new(x, z), self.noise_gen);
                self.chunks.insert(xz_to_index(x, z), chunk);
            }
        }
    }

    pub fn mesh_chunks(&mut self, device: &wgpu::Device) {
        for x in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
            for z in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
                let index = xz_to_index(x, z);

                self.mesh_chunk(device, index);
            }
        }
    }

    pub fn mesh_slice(&self, device: &wgpu::Device, chunk: &Chunk, y_slice: u32) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let t = Stopwatch::start_new();
        let mut vertices: Vec<SurfaceVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let pos = Vector3::new(chunk.position.x * 16, 0, chunk.position.y * 16);
        let rel_abs_x = chunk.position.x * 16;
        let rel_abs_z = chunk.position.y * 16;

        for x in 0..16 {
            for z in 0..16 {
                for y in y_slice * 16..(y_slice + 1) * 16 {
                    let block_at = chunk.get_block_at(x, y, z);

                    let current = block_at;

                    if current.has_partial_transparency() || !current.does_mesh() {continue;};

                    let front = get_block_at_absolute((x as i32) + rel_abs_x, y as i32, (z as i32) + rel_abs_z + 1, &self.chunks);
                    let back = get_block_at_absolute((x as i32) + rel_abs_x, y as i32, (z as i32) + rel_abs_z - 1, &self.chunks);
                    let up = get_block_at_absolute((x as i32) + rel_abs_x, (y as i32) + 1, (z as i32) + rel_abs_z, &self.chunks);
                    let down = get_block_at_absolute((x as i32) + rel_abs_x, (y as i32) - 1, (z as i32) + rel_abs_z, &self.chunks);
                    let right = get_block_at_absolute((x as i32) + rel_abs_x + 1, y as i32, (z as i32) + rel_abs_z, &self.chunks);
                    let left = get_block_at_absolute((x as i32) + rel_abs_x - 1, y as i32, (z as i32) + rel_abs_z, &self.chunks);
                    
                    if front.is_some() && front.unwrap().has_partial_transparency() {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [0, 1, 2, 1, 3, 2]);

                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Front, 0, current.get_surface_textures(BlockFace::Front)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Front, 1, current.get_surface_textures(BlockFace::Front)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Front, 2, current.get_surface_textures(BlockFace::Front)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Front, 3, current.get_surface_textures(BlockFace::Front)));
                    }

                    if back.is_some() && back.unwrap().has_partial_transparency() {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [2, 1, 0, 2, 3, 1]);

                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32) as f32], BlockFace::Back, 0, current.get_surface_textures(BlockFace::Back)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32) as f32], BlockFace::Back, 1, current.get_surface_textures(BlockFace::Back)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32) as f32], BlockFace::Back, 2, current.get_surface_textures(BlockFace::Back)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32) as f32], BlockFace::Back, 3, current.get_surface_textures(BlockFace::Back)));
                    }

                    if right.is_some() && right.unwrap().has_partial_transparency() {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [2, 1, 0, 2, 3, 1]);

                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32) as f32], BlockFace::Right, 0, current.get_surface_textures(BlockFace::Right)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Right, 1, current.get_surface_textures(BlockFace::Right)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32) as f32], BlockFace::Right, 2, current.get_surface_textures(BlockFace::Right)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Right, 3, current.get_surface_textures(BlockFace::Right)));
                    }

                    if left.is_some() && left.unwrap().has_partial_transparency() {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [0, 1, 2, 1, 3, 2]);

                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32) as f32], BlockFace::Left, 0, current.get_surface_textures(BlockFace::Left)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Left, 1, current.get_surface_textures(BlockFace::Left)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32) as f32], BlockFace::Left, 2, current.get_surface_textures(BlockFace::Left)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Left, 3, current.get_surface_textures(BlockFace::Left)));
                    }

                    if up.is_some() && up.unwrap().has_partial_transparency() {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [0, 1, 2, 1, 3, 2]);

                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32) as f32], BlockFace::Top, 0, current.get_surface_textures(BlockFace::Top)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Top, 1, current.get_surface_textures(BlockFace::Top)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32) as f32], BlockFace::Top, 2, current.get_surface_textures(BlockFace::Top)));
                        vertices.push(SurfaceVertex::from_position([(pos.x + x as i32 + 1) as f32, (pos.y + y as i32 + 1) as f32, (pos.z + z as i32 + 1) as f32], BlockFace::Top, 3, current.get_surface_textures(BlockFace::Top)));
                    }

                    if down.is_some() && down.unwrap().has_partial_transparency() {
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
            label: Some(&format!("Chunk Index Buffer")),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        println!("Took {}ms to mesh", t.elapsed_ms());
        //todo: find a faster and better way to mesh.
        (vertex_buffer, index_buffer, ilen)
    }

    pub fn mesh_chunk(&mut self, device: &wgpu::Device, index: u32) {
        let chunk = self.chunks.get(&index).unwrap();

        let slices = (0..16).into_iter();

        let buffers = slices.map(|s| {
            self.mesh_slice(device, chunk, s)
        }).collect::<Vec<(wgpu::Buffer, wgpu::Buffer, u32)>>();

        let rechunk = self.chunks.get_mut(&index).unwrap();
        
        rechunk.set_solid_buffers(buffers);
    }
}