use std::{collections::{HashMap, VecDeque}, sync::{Arc, RwLock}};

use cgmath::{Vector2, Vector3};
use noise::Perlin;
use rand::{RngCore, SeedableRng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use stopwatch::Stopwatch;
use wgpu::util::DeviceExt;

use crate::{blocks::{airblock::AirBlock, block::{calculate_illumination_bytes, Block, BlockFace, BlockType}}, engine::surfacevertex::SurfaceVertex, vox::chunkactionqueue::ChunkAction};

use super::{chunk::{local_xyz_to_index, xz_to_index, Chunk}, chunkactionqueue::ChunkActionQueue};

pub struct ChunkManager {
    pub chunks: HashMap<u32, Chunk>,
    render_distance: u32,
    seed: u32,
    noise_gen: Perlin,
    pub action_queue: ChunkActionQueue
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
            render_distance: 10,
            seed: 52352,
            noise_gen: Perlin::new(rand::rngs::StdRng::seed_from_u64(52352).next_u32()),
            action_queue: ChunkActionQueue::new()
        }
    }

    pub fn on_frame_action(&mut self, device: &wgpu::Device) {
        const MAX_ACTIONS: u32 = 15;

        for i in 0..MAX_ACTIONS {
            let res = self.action_queue.get_next_action();
            if res.is_none() {break;}

            match res.unwrap() {
                ChunkAction::BreakBlock(pos) => {
                    self.break_block(device, pos.x, pos.y as u32, pos.z)
                },
                ChunkAction::PlaceBlock(block) => {
                    self.place_block(device, block)
                }
            }
        }
    }

    pub fn generate_chunks(&mut self, device: &wgpu::Device) {
        let t = Stopwatch::start_new();
        for x in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
            for z in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
                let chunk = Chunk::new(device, Vector2::new(x, z), self.noise_gen);
                self.chunks.insert(xz_to_index(x, z), chunk);
            }
        }
        println!("Took {} seconds to generate all", t.elapsed_ms() / 1000);
    }
 
    pub fn generate_chunk_illumination(&mut self) {
        let t = Stopwatch::start_new();
        for x in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
            for z in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
                let index = xz_to_index(x, z);

                self.flood_lights(index);
            }
        }
        println!("Took {} seconds to illuminate all", t.elapsed_ms() / 1000);
    }

    pub fn mesh_chunks(&mut self, device: &wgpu::Device) {
        let t = Stopwatch::start_new();
        for x in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
            for z in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
                let index = xz_to_index(x, z);

                self.mesh_chunk(device, index);
            }
        }
        println!("Took {} seconds to mesh all", t.elapsed_ms() / 1000);
    }

    pub fn mesh_slice(&self, device: &wgpu::Device, chunk: &Chunk, y_slice: u32) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let mut vertices: Vec<SurfaceVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let rel_abs_x = chunk.position.x * 16;
        let rel_abs_z = chunk.position.y * 16;

        for x in 0..16 {
            for z in 0..16 {
                for yt in y_slice * 16..(y_slice + 1) * 16 {
                    let y = yt % 16;
                    let block_at = chunk.get_block_at(x, yt, z);

                    let current = block_at;

                    if current.has_partial_transparency() || !current.does_mesh() {continue;};

                    let front = get_block_at_absolute((x as i32) + rel_abs_x, yt as i32, (z as i32) + rel_abs_z + 1, &self.chunks);
                    let back = get_block_at_absolute((x as i32) + rel_abs_x, yt as i32, (z as i32) + rel_abs_z - 1, &self.chunks);
                    let up = get_block_at_absolute((x as i32) + rel_abs_x, (yt as i32) + 1, (z as i32) + rel_abs_z, &self.chunks);
                    let down = get_block_at_absolute((x as i32) + rel_abs_x, (yt as i32) - 1, (z as i32) + rel_abs_z, &self.chunks);
                    let right = get_block_at_absolute((x as i32) + rel_abs_x + 1, yt as i32, (z as i32) + rel_abs_z, &self.chunks);
                    let left = get_block_at_absolute((x as i32) + rel_abs_x - 1, yt as i32, (z as i32) + rel_abs_z, &self.chunks);

                    let illumination = calculate_illumination_bytes(block_at);

                    if front.is_some() && front.unwrap().has_partial_transparency() {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [0, 1, 2, 1, 3, 2]);

                        vertices.push(SurfaceVertex::from_position([x, y, z + 1], BlockFace::Front, 0, current.get_surface_textures(BlockFace::Front), illumination));
                        vertices.push(SurfaceVertex::from_position([x + 1 , y , z + 1 ], BlockFace::Front, 1, current.get_surface_textures(BlockFace::Front), illumination));
                        vertices.push(SurfaceVertex::from_position([x , y + 1 , z + 1 ], BlockFace::Front, 2, current.get_surface_textures(BlockFace::Front), illumination));
                        vertices.push(SurfaceVertex::from_position([x + 1 , y + 1 , z + 1 ], BlockFace::Front, 3, current.get_surface_textures(BlockFace::Front), illumination));
                    }

                    if back.is_some() && back.unwrap().has_partial_transparency() {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [2, 1, 0, 2, 3, 1]);

                        vertices.push(SurfaceVertex::from_position([x , y , z ], BlockFace::Back, 0, current.get_surface_textures(BlockFace::Back), illumination));
                        vertices.push(SurfaceVertex::from_position([x + 1 , y , z ], BlockFace::Back, 1, current.get_surface_textures(BlockFace::Back), illumination));
                        vertices.push(SurfaceVertex::from_position([x , y + 1 , z ], BlockFace::Back, 2, current.get_surface_textures(BlockFace::Back), illumination));
                        vertices.push(SurfaceVertex::from_position([x + 1 , y + 1 , z ], BlockFace::Back, 3, current.get_surface_textures(BlockFace::Back), illumination));
                    }

                    if right.is_some() && right.unwrap().has_partial_transparency() {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [2, 1, 0, 2, 3, 1]);

                        vertices.push(SurfaceVertex::from_position([x + 1 , y , z ], BlockFace::Right, 0, current.get_surface_textures(BlockFace::Right), illumination));
                        vertices.push(SurfaceVertex::from_position([x + 1 , y , z + 1 ], BlockFace::Right, 1, current.get_surface_textures(BlockFace::Right), illumination));
                        vertices.push(SurfaceVertex::from_position([x + 1 , y + 1 , z ], BlockFace::Right, 2, current.get_surface_textures(BlockFace::Right), illumination));
                        vertices.push(SurfaceVertex::from_position([x + 1 , y + 1 , z + 1 ], BlockFace::Right, 3, current.get_surface_textures(BlockFace::Right), illumination));
                    }

                    if left.is_some() && left.unwrap().has_partial_transparency() {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [0, 1, 2, 1, 3, 2]);

                        vertices.push(SurfaceVertex::from_position([x , y , z ], BlockFace::Left, 0, current.get_surface_textures(BlockFace::Left), illumination));
                        vertices.push(SurfaceVertex::from_position([x , y , z + 1 ], BlockFace::Left, 1, current.get_surface_textures(BlockFace::Left), illumination));
                        vertices.push(SurfaceVertex::from_position([x , y + 1 , z ], BlockFace::Left, 2, current.get_surface_textures(BlockFace::Left), illumination));
                        vertices.push(SurfaceVertex::from_position([x , y + 1 , z + 1 ], BlockFace::Left, 3, current.get_surface_textures(BlockFace::Left), illumination));
                    }

                    if up.is_some() && up.unwrap().has_partial_transparency() {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [0, 1, 2, 1, 3, 2]);

                        vertices.push(SurfaceVertex::from_position([x , y + 1 , z ], BlockFace::Top, 0, current.get_surface_textures(BlockFace::Top), illumination));
                        vertices.push(SurfaceVertex::from_position([x , y + 1 , z + 1 ], BlockFace::Top, 1, current.get_surface_textures(BlockFace::Top), illumination));
                        vertices.push(SurfaceVertex::from_position([x + 1 , y + 1 , z ], BlockFace::Top, 2, current.get_surface_textures(BlockFace::Top), illumination));
                        vertices.push(SurfaceVertex::from_position([x + 1 , y + 1 , z + 1 ], BlockFace::Top, 3, current.get_surface_textures(BlockFace::Top), illumination));
                    }

                    if down.is_some() && down.unwrap().has_partial_transparency() {
                        let current_l = vertices.len();
                        push_n(&mut indices, current_l as u32, [2, 1, 0, 2, 3, 1]);

                        vertices.push(SurfaceVertex::from_position([x , y , z ], BlockFace::Bottom, 0, current.get_surface_textures(BlockFace::Bottom), illumination));
                        vertices.push(SurfaceVertex::from_position([x , y , z + 1 ], BlockFace::Bottom, 1, current.get_surface_textures(BlockFace::Bottom), illumination));
                        vertices.push(SurfaceVertex::from_position([x + 1 , y , z ], BlockFace::Bottom, 2, current.get_surface_textures(BlockFace::Bottom), illumination));
                        vertices.push(SurfaceVertex::from_position([x + 1 , y , z + 1 ], BlockFace::Bottom, 3, current.get_surface_textures(BlockFace::Bottom), illumination));
                    }

                }
            }
        }

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

        //todo: find a faster and better way to mesh.
        (vertex_buffer, index_buffer, ilen)
    }

    pub fn get_block_at_absolute(&self, x: i32, y: i32, z: i32) -> Option<&BlockType> {
        if y < 0 || y > 255 {return None};
        let chunk_x = x.div_euclid(16);
        let chunk_z = z.div_euclid(16);

        let chunk = self.chunks.get(&xz_to_index(chunk_x, chunk_z))?;

        Some(chunk.get_block_at(x as u32, y as u32, z as u32))
    }

    pub fn break_block(&mut self, device: &wgpu::Device, x: i32, y: u32, z: i32) {
        let index = xz_to_index(x.div_euclid(16), z.div_euclid(16));
        let chunk = self.chunks.get_mut(&index).unwrap();

        let xrem = x.rem_euclid(16) as u32;
        let zrem = z.rem_euclid(16) as u32;
        let yrem = y % 16;

        //TODO: do removal formalities, such as dropping the block...

        chunk.grid[(y / 16) as usize][local_xyz_to_index(xrem, yrem, zrem) as usize] = Box::new(
            AirBlock::new(
                Vector3::new(xrem, yrem, zrem), 
                Vector3::new(x, y as i32, z)
            )
        );

        let xd = x.div_euclid(16);
        let zd = z.div_euclid(16);
        let yd = y.div_euclid(16);
        //gets the adjacent chunks, including itself :)
        let requires_meshing = (xd - 1..=xd + 1).map(|x| {
            (zd - 1..=zd + 1).map(move |z| {
                (yd - 1..=yd + 1).map(move |y| {
                    (x, y, z)
                })
                
            }).flatten()
        }).flatten();

        let t = Stopwatch::start_new();
        requires_meshing.for_each(|v| {
            let index = xz_to_index(v.0, v.2);
            let slice = v.1;

            if slice >= 16 {return};

            let chunk = self.chunks.get(&index);
            if chunk.is_none() {return};

            self.flood_lights(index);

            let chunk = self.chunks.get(&index).unwrap();
            let t = Stopwatch::start_new();
            let buffers = self.mesh_slice(device, chunk, slice as u32);
            
            let chunk = self.chunks.get_mut(&index).unwrap();
            chunk.set_solid_buffer(slice as u32, buffers);
            println!("{}ms for 1", t.elapsed_ms());
        });
        println!("regeneration took {}ms", t.elapsed_ms());
    }

    pub fn place_block(&mut self, device: &wgpu::Device, block: BlockType) {
        let abs = block.get_absolute_position();

        let index = xz_to_index(abs.x.div_euclid(16), abs.z.div_euclid(16));

        let local = block.get_relative_position();

        let chunk = self.chunks.get_mut(&index).unwrap();

        chunk.grid[(abs.y / 16) as usize][local_xyz_to_index(local.x, local.y, local.z) as usize] = block;

        let xd = abs.x.div_euclid(16);
        let zd = abs.z.div_euclid(16);
        let yd = abs.y.div_euclid(16);
        //gets the adjacent chunks, including itself :)
        let requires_meshing = (xd - 1..=xd + 1).map(|x| {
            (zd - 1..=zd + 1).map(move |z| {
                (yd - 1..=yd + 1).map(move |y| {
                    (x, y, z)
                })
                
            }).flatten()
        }).flatten();

        let t = Stopwatch::start_new();
        requires_meshing.for_each(|v| {
            let index = xz_to_index(v.0, v.2);
            let slice = v.1;

            if slice >= 16 {return};

            let chunk = self.chunks.get(&index);
            if chunk.is_none() {return};

            self.flood_lights(index);

            let chunk = self.chunks.get(&index).unwrap();

            let buffers = self.mesh_slice(device, chunk, slice as u32);

            let chunk = self.chunks.get_mut(&index).unwrap();
            chunk.set_solid_buffer(slice as u32, buffers);
        });
        println!("regeneration took {}ms", t.elapsed_ms());
    }

    pub fn flood_lights(&mut self, chunk_index: u32) {
        let chunk = self.chunks.get_mut(&chunk_index).unwrap();
        for x in 0..16 {
            for z in 0..16 {
                for y in (0..256).rev() {
                    //guaranteed to exist.

                    let block = chunk.get_block_at(x, y, z);

                    //if it is the first solid block hit...
                    if !block.has_partial_transparency() {
                        //start spreading light downwards...
                        for sy in (y - 15)..=y {
                            chunk.modify_block_at(x as u32, sy as u32, z as u32, |block| {
                                block.set_sunlight_intensity((15 - (y - sy)) as u8);
                            });
                        }
                        break;
                    }
                }
            }
        }

        //let queue: VecDeque<BlockType> = VecDeque::new();

        //for emissive colors
        // for x in 0..16 {
        //     for z in 0..16 {
        //         for y in slice * 16..(slice + 1) * 16 {
        //             let block = get_block_at_absolute(x, y as i32, z, &self.chunks);

                    
        //         }
        //     }
        // }
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