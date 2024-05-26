use std::{collections::{HashMap, VecDeque}, sync::{Arc, RwLock}};

use cgmath::{InnerSpace, Vector2, Vector3};
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
    pub action_queue: ChunkActionQueue,
    update_queue: ChunkActionQueue
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
            action_queue: ChunkActionQueue::new(),
            update_queue: ChunkActionQueue::new()
        }
    }

    pub fn on_frame_action(&mut self, device: &wgpu::Device) {
        let t = Stopwatch::start_new();
        const MAX_ACTIONS: u32 = 15;
        for _ in 0..MAX_ACTIONS {
            let res = self.action_queue.get_next_action();
            if res.is_none() {break;}

            let u = res.unwrap();
            match u {
                ChunkAction::BreakBlock(pos) => {
                    self.break_block(device, pos.x, pos.y as u32, pos.z)
                },
                ChunkAction::PlaceBlock(block) => {
                    self.place_block(device, block)
                },
                _ => {panic!("{:?} in wrong queue(action)", u)}
            }
        }
        
        const MAX_UPDATES: u32 = 7; //pretty good number for most devices? probably?
        
        for _ in 0..MAX_UPDATES {
            let res = self.update_queue.get_next_action();
            if res.is_none() {break;}

            let u = res.unwrap();

            match u {
                ChunkAction::UpdateChunkMesh(p) => {
                    let ind = xz_to_index(p.x, p.z);
                    let mesh = self.mesh_slice(device, self.chunks.get(&ind).unwrap(), p.y as u32);

                    let chunk = self.chunks.get_mut(&ind).unwrap();
                    
                    chunk.set_solid_buffer(p.y as u32, mesh.0);
                    chunk.set_transparent_buffer(p.y as u32, mesh.1);
                },
                ChunkAction::UpdateChunkLighting(p) => {
                    let ind = xz_to_index(p.x, p.y);
                    self.flood_lights(ind);
                },
                _ => {panic!("{:?} in wrong queue(update)", u)}
            }
        }
        println!("FRAME: {}ms", t.elapsed_ms());
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

    pub fn mesh_slice(&self, device: &wgpu::Device, chunk: &Chunk, y_slice: u32) -> ((wgpu::Buffer, wgpu::Buffer, u32), (wgpu::Buffer, wgpu::Buffer, u32)) {
        let mut vertices: Vec<SurfaceVertex> = Vec::with_capacity(16 * 16 * 16 * 6 * 4);
        let mut indices: Vec<u32> = Vec::with_capacity(16 * 16 * 16 * 6 * 6);
        let rel_abs_x = chunk.position.x * 16;
        let rel_abs_z = chunk.position.y * 16;
        let y_start = y_slice * 16;
        let y_end = (y_slice + 1) * 16;
        let mut vertices_transparent: Vec<SurfaceVertex> = Vec::with_capacity(16 * 16 * 16 * 6 * 4);
        let mut indices_transparent: Vec<u32> = Vec::with_capacity(16 * 16 * 16 * 6 * 6);

        for x in 0..16 {
            for z in 0..16 {
                let abs_x = x as i32 + rel_abs_x;
                let abs_z = z as i32 + rel_abs_z;

                for yt in y_start..y_end {
                    let y = yt % 16;
                    let block_at = chunk.get_block_at(x, yt, z);

                    if !block_at.does_mesh() {
                        continue;
                    }

                    let illumination = calculate_illumination_bytes(block_at);

                    let neighbors = [
                        get_block_at_absolute(abs_x, yt as i32, abs_z + 1, &self.chunks),
                        get_block_at_absolute(abs_x, yt as i32, abs_z - 1, &self.chunks),
                        get_block_at_absolute(abs_x + 1, yt as i32, abs_z, &self.chunks),
                        get_block_at_absolute(abs_x - 1, yt as i32, abs_z, &self.chunks),
                        get_block_at_absolute(abs_x, yt as i32 + 1, abs_z, &self.chunks),
                        get_block_at_absolute(abs_x, yt as i32 - 1, abs_z, &self.chunks),
                    ];

                    let faces = [
                        BlockFace::Front,
                        BlockFace::Back,
                        BlockFace::Right,
                        BlockFace::Left,
                        BlockFace::Top,
                        BlockFace::Bottom,
                    ];

                    for (i, neighbor) in neighbors.iter().enumerate() {
                        if let Some(neighbor_block) = neighbor {
                            if neighbor_block.has_partial_transparency() {
                                let current_l = vertices.len() as u32;
                                let face = faces[i];

                                let (face_vertices, face_indices) = match face {
                                    BlockFace::Front => (
                                        [
                                            [x, y, z + 1],
                                            [x + 1, y, z + 1],
                                            [x, y + 1, z + 1],
                                            [x + 1, y + 1, z + 1],
                                        ],
                                        [0, 1, 2, 1, 3, 2],
                                    ),
                                    BlockFace::Back => (
                                        [
                                            [x, y, z],
                                            [x + 1, y, z],
                                            [x, y + 1, z],
                                            [x + 1, y + 1, z],
                                        ],
                                        [2, 1, 0, 2, 3, 1],
                                    ),
                                    BlockFace::Right => (
                                        [
                                            [x + 1, y, z],
                                            [x + 1, y, z + 1],
                                            [x + 1, y + 1, z],
                                            [x + 1, y + 1, z + 1],
                                        ],
                                        [2, 1, 0, 2, 3, 1],
                                    ),
                                    BlockFace::Left => (
                                        [
                                            [x, y, z],
                                            [x, y, z + 1],
                                            [x, y + 1, z],
                                            [x, y + 1, z + 1],
                                        ],
                                        [0, 1, 2, 1, 3, 2],
                                    ),
                                    BlockFace::Top => (
                                        [
                                            [x, y + 1, z],
                                            [x, y + 1, z + 1],
                                            [x + 1, y + 1, z],
                                            [x + 1, y + 1, z + 1],
                                        ],
                                        [0, 1, 2, 1, 3, 2],
                                    ),
                                    BlockFace::Bottom => (
                                        [
                                            [x, y, z],
                                            [x, y, z + 1],
                                            [x + 1, y, z],
                                            [x + 1, y, z + 1],
                                        ],
                                        [2, 1, 0, 2, 3, 1],
                                    ),
                                };

                                if block_at.has_partial_transparency() {
                                    indices_transparent.extend(face_indices.iter().map(|&index| index + current_l));
                                    for (j, &pos) in face_vertices.iter().enumerate() {
                                        vertices_transparent.push(SurfaceVertex::from_position(
                                            pos, face, j as u32, block_at.get_surface_textures(face), illumination
                                        ));
                                    }
                                }
                                else {
                                    indices.extend(face_indices.iter().map(|&index| index + current_l));
                                    for (j, &pos) in face_vertices.iter().enumerate() {
                                        vertices.push(SurfaceVertex::from_position(
                                            pos, face, j as u32, block_at.get_surface_textures(face), illumination
                                        ));
                                    }
                                }
                            }
                        }
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

        let ilen_t = indices_transparent.len() as u32;

        let vertex_buffer_t = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Chunk Vertex Buffer Transparent")),
            contents: bytemuck::cast_slice(&vertices_transparent),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer_t = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Chunk Index Buffer Transparent")),
            contents: bytemuck::cast_slice(&indices_transparent),
            usage: wgpu::BufferUsages::INDEX,
        });

        ((vertex_buffer, index_buffer, ilen), ((vertex_buffer_t, index_buffer_t, ilen_t)))
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
        let mut requires_meshing = (xd - 1..=xd + 1).map(|x| {
            (zd - 1..=zd + 1).map(move |z| {
                (0..16).map(move |y| {
                    Vector3::new(x as f32, y as f32, z as f32)
                })
                
            }).flatten()
        }).flatten().collect::<Vec<Vector3<f32>>>();

        let requires_meshing_light = (xd - 1..=xd + 1).map(|x| {
            (zd - 1..=zd + 1).map(move |z| {
                (x, z)
            })
        }).flatten();

        requires_meshing_light.for_each(|v| {
            let index = xz_to_index(v.0, v.1);

            let chunk = self.chunks.get(&index);
            if chunk.is_none() {return};

            self.update_queue.update_chunk_lighting(Vector2::new(v.0, v.1));
        });

        let xyz: Vector3<f32> = Vector3::new(
            if xrem == 0 {xd - 1} else if xrem == 15 {xd + 1} else {xd} as f32,
            if yrem == 0 {yd - 1} else if yrem == 15 {yd + 1} else {yd} as f32,
            if zrem == 0 {zd - 1} else if zrem == 15 {zd + 1} else {zd} as f32,
        );

        requires_meshing.sort_by(|a, b| (a - xyz).magnitude().partial_cmp(&(b - xyz).magnitude()).unwrap());

        for v in requires_meshing {
            self.update_queue.update_chunk_mesh(Vector3::new(v.x as i32, v.y as i32, v.z as i32));
        }
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
        let mut requires_meshing = (xd - 1..=xd + 1).map(|x| {
            (zd - 1..=zd + 1).map(move |z| {
                (0..16).map(move |y| {
                    Vector3::new(x as f32, y as f32, z as f32)
                })
                
            }).flatten()
        }).flatten().collect::<Vec<Vector3<f32>>>();

        let requires_meshing_light = (xd - 1..=xd + 1).map(|x| {
            (zd - 1..=zd + 1).map(move |z| {
                (x, z)
            })
        }).flatten();

        requires_meshing_light.for_each(|v| {
            let index = xz_to_index(v.0, v.1);

            let chunk = self.chunks.get(&index);
            if chunk.is_none() {return};

            self.update_queue.update_chunk_lighting(Vector2::new(v.0, v.1));
        });

        let xyz: Vector3<f32> = Vector3::new(
            if local.x == 0 {xd - 1} else if local.x == 15 {xd + 1} else {xd} as f32,
            if local.y == 0 {yd - 1} else if local.y == 15 {yd + 1} else {yd} as f32,
            if local.z == 0 {zd - 1} else if local.z == 15 {zd + 1} else {zd} as f32,
        );

        requires_meshing.sort_by(|a, b| (a - xyz).magnitude().partial_cmp(&(b - xyz).magnitude()).unwrap());

        for v in requires_meshing {
            self.update_queue.update_chunk_mesh(Vector3::new(v.x as i32, v.y as i32, v.z as i32));
        }
    }

    pub fn flood_lights(&mut self, chunk_index: u32) {
        let chunk = self.chunks.get_mut(&chunk_index).unwrap();
        for x in 0..16 {
            for z in 0..16 {
                let mut initial_height = 1000; //safe value
                for y in (0..256).rev() {
                    if initial_height != 1000 && y < initial_height - 15 {
                        chunk.modify_block_at(x as u32, y as u32, z as u32, |block| {
                            block.set_sunlight_intensity(0);
                        });
                        continue;
                    }
                    else if initial_height != 1000 {
                        continue;
                    }

                    let block = chunk.get_block_at(x, y, z);

                    //if it is the first solid block hit...
                    if !block.has_partial_transparency() {
                        //start spreading light downwards...
                        for sy in (y - 15)..=y {
                            chunk.modify_block_at(x as u32, sy as u32, z as u32, |block| {
                                block.set_sunlight_intensity((15 - (y - sy)) as u8);
                            });
                        }
                        initial_height = y;
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
        
        let mut v0: Vec<(wgpu::Buffer, wgpu::Buffer, u32)> = Vec::new();
        let mut v1: Vec<(wgpu::Buffer, wgpu::Buffer, u32)> = Vec::new();

        slices.for_each(|s| {
            let out = self.mesh_slice(device, chunk, s);
            v0.push(out.0);
            v1.push(out.1);
        });

        let rechunk = self.chunks.get_mut(&index).unwrap();
        
        rechunk.set_solid_buffers(v0);
        rechunk.set_transparent_buffers(v1);
    }
}