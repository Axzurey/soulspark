use std::{collections::HashMap, mem, sync::{Arc, RwLock}};

use cached::proc_macro::cached;
use cgmath::{Vector2, Vector3};
use noise::Perlin;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use stopwatch::Stopwatch;
use wgpu::util::DeviceExt;

use crate::{blocks::{airblock::AirBlock, block::{Block, BlockType, Blocks}, dirtblock::DirtBlock, grassblock::GrassBlock, stoneblock::StoneBlock}, engine::vertex::{ModelVertex, Vertex}, vox::{structure_loader::get_blocks_for_structure_at_point, worldgen::density_map_plane}};

use super::worldgen::generate_surface_height;

#[cached]
pub fn local_xyz_to_index(x: u32, y: u32, z: u32) -> u32 {
    ((z * 16 * 16) + (y * 16) + x) as u32
}

#[cached]
pub fn xz_to_index(x: i32, z: i32) -> u32 {
    let x0 = if x >= 0 {2 * x} else {-2 * x - 1}; //converting integers to natural numbers
    let z0 = if z >= 0 {2 * z} else {-2 * z - 1};

    (0.5 * (x0 + z0) as f32 * (x0 + z0 + 1) as f32 + z0 as f32) as u32 //cantor pairing https://math.stackexchange.com/questions/3003672/convert-infinite-2d-plane-integer-coords-to-1d-number
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkDataVertex {
    pub position_sliced: [i32; 3]
}

impl Vertex for ChunkDataVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ChunkDataVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Sint32x3,
                },
            ],
        }
    }
}

pub struct Chunk {
    pub position: Vector2<i32>,
    pub grid: Vec<Vec<BlockType>>,
    
    //(vertex, index, len_indices)
    solid_buffers: Vec<(wgpu::Buffer, wgpu::Buffer, u32)>,
    transparent_buffers: Vec<(wgpu::Buffer, wgpu::Buffer, u32)>,
    pub slice_vertex_buffers: Vec<wgpu::Buffer> 
}

impl Chunk {
    pub fn new(device: &wgpu::Device, position: Vector2<i32>, noisegen: Perlin, extra_blocks: &mut HashMap<u32, Vec<BlockType>>) -> Self {
        let t = Stopwatch::start_new();

        let iter_layers = (0..16).into_iter();

        let mut extra_blocks_same: Vec<BlockType> = Vec::new();

        let mut blocks = iter_layers.map(|y_slice| {
            let mut out: Vec<BlockType> = Vec::with_capacity(4096);

            let uninit = out.spare_capacity_mut();

            for x in 0..16 {
                for z in 0..16 {
                    let abs_x = ((x as i32) + position.x * 16) as i32;
                    let abs_z = ((z as i32) + position.y * 16) as i32;

                    let floor_level = generate_surface_height(noisegen, abs_x, abs_z);
                    for y in 0..16 {
                        let abs_y = (y + y_slice as u32 * 16) as i32;

                        let block: BlockType =
                        if abs_y == floor_level && abs_y < 100 {
                            Box::new(GrassBlock::new(
                                Vector3::new(x, y as u32, z), 
                                Vector3::new(abs_x, abs_y, abs_z))
                            )
                        }
                        else if abs_y + 3 < floor_level || (abs_y == floor_level && abs_y >= 100) {
                            Box::new(StoneBlock::new(
                                Vector3::new(x, y as u32, z), 
                                Vector3::new(abs_x, abs_y, abs_z))
                            )
                        }
                        else if abs_y < floor_level {
                            if abs_y < 100 {
                                Box::new(DirtBlock::new(
                                    Vector3::new(x, y as u32, z), 
                                    Vector3::new(abs_x, abs_y, abs_z))
                                )
                            }
                            else {
                                Box::new(StoneBlock::new(
                                    Vector3::new(x, y as u32, z), 
                                    Vector3::new(abs_x, abs_y, abs_z))
                                )
                            }
                        }
                        else {
                            Box::new(AirBlock::new(
                                Vector3::new(x, y as u32, z), 
                                Vector3::new(abs_x, abs_y, abs_z))
                            )
                        };

                        if abs_y == floor_level + 1 {
                            let should_tree = density_map_plane(noisegen, abs_x, abs_z);

                            if should_tree {
                                let blocks = get_blocks_for_structure_at_point("tree", 0, Vector3::new(abs_x, abs_y, abs_z));

                                extra_blocks_same.extend(blocks);
                            }
                        }

                        uninit[local_xyz_to_index(x, y as u32, z) as usize].write(block);
                    }
                }
            }

            unsafe { out.set_len(4096) };

            out
        }).collect::<Vec<Vec<BlockType>>>();

        for block in extra_blocks_same {
            if block.get_block() == Blocks::AIR {continue};

            let p = block.get_absolute_position();

            if p.x.div_euclid(16) == position.x && p.z.div_euclid(16) == position.y {
                let rel = block.get_relative_position();
                blocks[p.y.div_euclid(16) as usize][local_xyz_to_index(rel.x, rel.y, rel.z) as usize] = block;
            }
        }

        let slice_vertex_buffers = (0..16).map(|y| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Chunk Data Buffer")),
                contents: bytemuck::cast_slice(&[ChunkDataVertex {
                    position_sliced: [position.x, y, position.y]
                }]),
                usage: wgpu::BufferUsages::VERTEX,
            })
        }).collect::<Vec<wgpu::Buffer>>();

        println!("Took {}ms to generate chunk", t.elapsed_ms());

        Self {
            position,
            grid: blocks,
            solid_buffers: Vec::new(),
            transparent_buffers: Vec::new(),
            slice_vertex_buffers
        }
    }

    pub fn get_block_at(&self, x: u32, y: u32, z: u32) -> &BlockType {
        &self.grid[(y / 16) as usize][local_xyz_to_index(x % 16, y % 16, z % 16) as usize]
    }

    pub fn set_solid_buffer(&mut self, slice: u32, buffers: (wgpu::Buffer, wgpu::Buffer, u32)) {
        self.solid_buffers[slice as usize] = buffers;
    }
    pub fn set_solid_buffers(&mut self, buffers: Vec<(wgpu::Buffer, wgpu::Buffer, u32)>) {
        self.solid_buffers = buffers;
    }
    pub fn get_solid_buffer(&self, slice: u32) -> &(wgpu::Buffer, wgpu::Buffer, u32) {
        &self.solid_buffers[slice as usize]
    }
    pub fn get_solid_buffers(&self) -> &Vec<(wgpu::Buffer, wgpu::Buffer, u32)> {
        &self.solid_buffers
    }
    pub fn set_transparent_buffer(&mut self, slice: u32, buffers: (wgpu::Buffer, wgpu::Buffer, u32)) {
        self.transparent_buffers[slice as usize] = buffers;
    }
    pub fn set_transparent_buffers(&mut self, buffers: Vec<(wgpu::Buffer, wgpu::Buffer, u32)>) {
        self.transparent_buffers = buffers;
    }
    pub fn get_transparent_buffer(&self, slice: u32) -> &(wgpu::Buffer, wgpu::Buffer, u32) {
        &self.transparent_buffers[slice as usize]
    }
    pub fn get_transparent_buffers(&self) -> &Vec<(wgpu::Buffer, wgpu::Buffer, u32)> {
        &self.transparent_buffers
    }

    pub fn modify_block_at<F>(&mut self, x: u32, y: u32, z: u32, mut callback: F) where F: FnMut(&mut BlockType) {
        callback(&mut self.grid[(y / 16) as usize][local_xyz_to_index(x % 16, y % 16, z % 16) as usize]);
    }
}