use std::sync::{Arc, RwLock};

use cached::proc_macro::cached;
use cgmath::{Vector2, Vector3};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use stopwatch::Stopwatch;

use crate::blocks::{airblock::AirBlock, block::Block, dirtblock::DirtBlock, grassblock::GrassBlock};

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

pub struct Chunk {
    pub position: Vector2<i32>,
    pub seed: u32,
    grid: [Vec<Arc<RwLock<dyn Block + Send + Sync>>>; 16],
    
    //(vertex, index, len_indices)
    solid_buffers: Vec<(wgpu::Buffer, wgpu::Buffer, u32)>,
}

impl Chunk {
    pub fn new(position: Vector2<i32>, seed: u32) -> Self {
        let t = Stopwatch::start_new();

        let iter_layers = (0..16).into_par_iter();

        let blocks = iter_layers.map(|y_slice| {
            let mut out: Vec<Arc<RwLock<dyn Block + Send + Sync>>> = Vec::with_capacity(4096);

            let uninit = out.spare_capacity_mut();

            for x in 0..16 {
                for z in 0..16 {
                    let abs_x = ((x as i32) + position.x * 16) as i32;
                    let abs_z = ((z as i32) + position.y * 16) as i32;

                    let floor_level = generate_surface_height(seed, abs_x, abs_z);
                    for y in 0..16 {
                        let abs_y = (y + y_slice as u32 * 16) as i32;

                        let block: Arc<RwLock<dyn Block + Send + Sync>> =
                        if abs_y == floor_level {
                            Arc::new(RwLock::new(GrassBlock::new(
                                Vector3::new(x, y as u32, z), 
                                Vector3::new(abs_x, abs_y, abs_z)))
                            )
                        }
                        else if abs_y < floor_level {
                            Arc::new(RwLock::new(DirtBlock::new(
                                Vector3::new(x, y as u32, z), 
                                Vector3::new(abs_x, abs_y, abs_z)))
                            )
                        }
                        else {
                            Arc::new(RwLock::new(AirBlock::new(
                                Vector3::new(x, y as u32, z), 
                                Vector3::new(abs_x, abs_y, abs_z)))
                            )
                        };

                        uninit[local_xyz_to_index(x, y as u32, z) as usize].write(block);
                    }
                }
            }

            unsafe { out.set_len(4096) };

            out
        }).collect::<Vec<Vec<Arc<RwLock<dyn Block + Send + Sync>>>>>();

        let out_size = blocks.len();

        let block_grid: [Vec<Arc<RwLock<dyn Block + Send + Sync>>>; 16] = blocks.try_into().expect(&format!("Error in chunk generator: BlockGrid length is not exactly 16, but rather, {}.", out_size));

        println!("Generated chunk in {}ms", t.elapsed_ms());

        Self {
            position,
            seed,
            grid: block_grid,
            solid_buffers: Vec::new()
        }
    }

    pub fn get_block_at(&self, x: u32, y: u32, z: u32) -> Arc<RwLock<dyn Block + Send + Sync>> {
        self.grid[(y / 16) as usize][local_xyz_to_index(x % 16, y % 16, z % 16) as usize].clone()
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
}