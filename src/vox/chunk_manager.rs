use std::{collections::{HashMap, HashSet, VecDeque}, sync::{mpsc::Sender, Arc}, thread};
use owning_ref::{OwningRef, RwLockReadGuardRef};
use parking_lot::{RwLock, RwLockReadGuard, };
use cgmath::{InnerSpace, MetricSpace, Vector2, Vector3};
use noise::Perlin;
use rand::{RngCore, SeedableRng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use stopwatch::Stopwatch;
use wgpu::util::DeviceExt;

use crate::{blocks::{airblock::AirBlock, block::{Block, BlockFace, BlockType, Blocks}}, engine::surfacevertex::SurfaceVertex, internal::depthsort::Quad, vox::chunkactionqueue::ChunkAction};

use super::{binarymesher::{binary_mesh, MeshStageType}, chunk::{local_xyz_to_index, xz_to_index, Chunk, ChunkBuffers, ChunkGridType, ChunkState}, chunkactionqueue::ChunkActionQueue};

#[derive(PartialEq)]
struct LightingBFSRemoveNode {
    pub position: Vector3<i32>,
    pub intensity: u8
}

#[derive(PartialEq)]
struct LightingBFSAddNode {
    pub position: Vector3<i32>
}

pub struct ChunkManager {
    pub chunks: HashMap<u32, Arc<Chunk>>,
    pub chunk_buffers: HashMap<u32, ChunkBuffers>,
    pub render_distance: u32,
    pub seed: u32,
    noise_gen: Perlin,
    pub action_queue: ChunkActionQueue,
    update_queue: ChunkActionQueue,
    extra_blocks: HashMap<u32, Vec<BlockType>>,
    pub unresolved_meshes: Vec<Vector3<i32>>
}

pub fn get_block_at_absolute(x: i32, y: i32, z: i32, chunks: &HashMap<u32, Arc<Chunk>>) -> Option<&BlockType> {
    if y < 0 || y > 255 {return None};
    let chunk_x = x.div_euclid(16);
    let chunk_z = z.div_euclid(16);

    chunks.get(&xz_to_index(chunk_x, chunk_z)).map(|v| v.get_block_at(x.rem_euclid(16) as u32, y as u32, z.rem_euclid(16) as u32))
}

pub fn mesh_slice_arrayed(chunk_x: i32, chunk_z: i32, y_slice: u32, chunks: &HashMap<u32, Arc<Chunk>>) -> ((Vec<SurfaceVertex>, Vec<u32>, u32), (Vec<SurfaceVertex>, Vec<u32>, u32, Vec<Quad>)) {
    //let chunk = &chunks[&xz_to_index(chunk_x, chunk_z)].read();
    
    // let mut vertices: Vec<SurfaceVertex> = Vec::with_capacity(16 * 16 * 16 * 6 * 4);
    // let mut indices: Vec<u32> = Vec::with_capacity(16 * 16 * 16 * 6 * 6);
    // let rel_abs_x = chunk_x * 16;
    // let rel_abs_z = chunk_z * 16;
    // let y_start = y_slice * 16;
    // let y_end = (y_slice + 1) * 16;
    // let mut vertices_transparent: Vec<SurfaceVertex> = Vec::with_capacity(16 * 16 * 16 * 6 * 4);
    // let mut indices_transparent: Vec<u32> = Vec::with_capacity(16 * 16 * 16 * 6 * 6);

    // //below is no longer needed due to read_recursive
    // //to prevent deadlocking(probably) :(
    // // let locks = (chunk_x - 1..=chunk_x + 1).flat_map(|x| {
    // //     (chunk_z - 1..=chunk_z + 1).map(|z| {
    // //         chunks.get(&xz_to_index(x, z)).map(|v| v.read_recursive())
    // //     })
    // // });

    // for x in 0..16 {
    //     for z in 0..16 {
    //         let abs_x = x as i32 + rel_abs_x;
    //         let abs_z = z as i32 + rel_abs_z;

    //         for yt in y_start..y_end {
    //             let y = yt % 16;

    //             let block_at = get_block_at_absolute(abs_x, yt as i32, abs_z, chunks).unwrap();

    //             if !block_at.does_mesh() || block_at.get_block() == Blocks::AIR {
    //                 continue;
    //             }
                
    //             let neighbors = [
    //                 get_block_at_absolute(abs_x, yt as i32, abs_z + 1, &chunks),
    //                 get_block_at_absolute(abs_x, yt as i32, abs_z - 1, &chunks),
    //                 get_block_at_absolute(abs_x + 1, yt as i32, abs_z, &chunks),
    //                 get_block_at_absolute(abs_x - 1, yt as i32, abs_z, &chunks),
    //                 get_block_at_absolute(abs_x, yt as i32 + 1, abs_z, &chunks),
    //                 get_block_at_absolute(abs_x, yt as i32 - 1, abs_z, &chunks),
    //             ];

    //             let faces = [
    //                 BlockFace::Front,
    //                 BlockFace::Back,
    //                 BlockFace::Right,
    //                 BlockFace::Left,
    //                 BlockFace::Top,
    //                 BlockFace::Bottom,
    //             ];

    //             let is_transparent = block_at.has_partial_transparency();

    //             let cb = block_at.get_block();

    //             for (i, neighbor) in neighbors.iter().enumerate() {
    //                 if let Some(neighbor_block) = neighbor {
    //                     if (neighbor_block.has_partial_transparency() && !is_transparent) || (is_transparent && cb != Blocks::AIR && cb != neighbor_block.get_block()) {
    //                         let current_l = if is_transparent {
    //                             vertices_transparent.len() as u32
    //                         } else {vertices.len() as u32};
    //                         let face = faces[i];

    //                         let (face_vertices, face_indices) = match face {
    //                             BlockFace::Front => (
    //                                 [
    //                                     [x, y, z + 1],
    //                                     [x + 1, y, z + 1],
    //                                     [x, y + 1, z + 1],
    //                                     [x + 1, y + 1, z + 1],
    //                                 ],
    //                                 [0, 1, 2, 1, 3, 2],
    //                             ),
    //                             BlockFace::Back => (
    //                                 [
    //                                     [x, y, z],
    //                                     [x + 1, y, z],
    //                                     [x, y + 1, z],
    //                                     [x + 1, y + 1, z],
    //                                 ],
    //                                 [2, 1, 0, 2, 3, 1],
    //                             ),
    //                             BlockFace::Right => (
    //                                 [
    //                                     [x + 1, y, z],
    //                                     [x + 1, y, z + 1],
    //                                     [x + 1, y + 1, z],
    //                                     [x + 1, y + 1, z + 1],
    //                                 ],
    //                                 [2, 1, 0, 2, 3, 1],
    //                             ),
    //                             BlockFace::Left => (
    //                                 [
    //                                     [x, y, z],
    //                                     [x, y, z + 1],
    //                                     [x, y + 1, z],
    //                                     [x, y + 1, z + 1],
    //                                 ],
    //                                 [0, 1, 2, 1, 3, 2],
    //                             ),
    //                             BlockFace::Top => (
    //                                 [
    //                                     [x, y + 1, z],
    //                                     [x, y + 1, z + 1],
    //                                     [x + 1, y + 1, z],
    //                                     [x + 1, y + 1, z + 1],
    //                                 ],
    //                                 [0, 1, 2, 1, 3, 2],
    //                             ),
    //                             BlockFace::Bottom => (
    //                                 [
    //                                     [x, y, z],
    //                                     [x, y, z + 1],
    //                                     [x + 1, y, z],
    //                                     [x + 1, y, z + 1],
    //                                 ],
    //                                 [2, 1, 0, 2, 3, 1],
    //                             ),
    //                         };

    //                         let illumination = calculate_illumination_bytes(neighbor_block);
    //                         if is_transparent {
    //                             indices_transparent.extend(face_indices.iter().map(|&index| index + current_l));
    //                             for (j, &pos) in face_vertices.iter().enumerate() {
    //                                 vertices_transparent.push(SurfaceVertex::from_position(
    //                                     pos, face, j as u32, block_at.get_surface_textures(face), illumination
    //                                 ));
    //                             }
    //                         }
    //                         else {
    //                             indices.extend(face_indices.iter().map(|&index| index + current_l));
    //                             for (j, &pos) in face_vertices.iter().enumerate() {
    //                                 vertices.push(SurfaceVertex::from_position(
    //                                     pos, face, j as u32, block_at.get_surface_textures(face), illumination
    //                                 ));
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    // let ilen = indices.len() as u32;
    // let itlen = indices_transparent.len() as u32;

    // (
    //     (vertices, indices, ilen),
    //     (vertices_transparent, indices_transparent, itlen)
    // )
    let solidmesh = binary_mesh(chunk_x, chunk_z, y_slice, chunks, MeshStageType::Solid);
    (
        (solidmesh.0, solidmesh.1, solidmesh.2),
        binary_mesh(chunk_x, chunk_z, y_slice, chunks, MeshStageType::Transparent)
    )
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
            chunk_buffers: HashMap::new(),
            render_distance: 5,
            seed: 6744464,
            noise_gen: Perlin::new(rand::rngs::StdRng::seed_from_u64(88).next_u32()),
            action_queue: ChunkActionQueue::new(),
            update_queue: ChunkActionQueue::new(),
            unresolved_meshes: Vec::new(),
            extra_blocks: HashMap::new()
        }
    }

    pub fn on_frame_action(&mut self, device: &wgpu::Device, chunk_send: &Sender<(i32, i32, u32, HashMap<u32, Arc<Chunk>>)>) {
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
        
        let mut remaining_updates: u32 = 1; //lighting updates per frame.(meshing is sent to another chunk)
        
        while remaining_updates > 0 {
            let res = self.update_queue.get_next_action();
            if res.is_none() {break;}

            let u = res.unwrap();
            match u {
                ChunkAction::UpdateChunkMesh(p) => {
                    if !self.unresolved_meshes.contains(&p) {
                        chunk_send.send((p.x, p.z, p.y as u32, self.chunks.clone())).unwrap();
                        self.unresolved_meshes.push(p);
                    }
                },
                ChunkAction::UpdateChunkLighting(p) => {
                    let ind = xz_to_index(p.x, p.y);
                    self.flood_lights(ind);
                    remaining_updates -= 1;
                },
                _ => {panic!("{:?} in wrong queue(update)", u)}
            }

        }

        //println!("FRAME: {}ms", t.elapsed_ms());
    }

    pub fn generate_chunks(&mut self, device: &wgpu::Device, send_queue: &Sender<(i32, i32)>, origin: Vector2<f32>) {
        let mut chunks = (-(self.render_distance as i32)..=(self.render_distance as i32)).flat_map(|x| {
            (-(self.render_distance as i32)..=(self.render_distance as i32)).map(move |z| {
                Vector2::new(x as f32, z as f32)
            })
        }).collect::<Vec<_>>();

        chunks.sort_by(|a, b| {a.distance(origin).partial_cmp(&b.distance(origin)).unwrap()});

        for chunk in chunks {
            let chunkbuffs = ChunkBuffers::new(chunk.x as i32, chunk.y as i32);
            self.chunk_buffers.insert(xz_to_index(chunk.x as i32, chunk.y as i32), chunkbuffs);
            send_queue.send((chunk.x as i32, chunk.y as i32)).unwrap();
        }
    }
 
    pub fn generate_chunk_illumination(&mut self, device: &wgpu::Device) {
        let t = Stopwatch::start_new();

        for x in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
            for z in -(self.render_distance as i32)..(self.render_distance + 1) as i32 {
                let index = xz_to_index(x, z);

                self.flood_lights(index);
            }
        }
        println!("Took {} seconds to illuminate all", t.elapsed_ms() / 1000);
    }

    pub fn mesh_chunks(&mut self, device: &wgpu::Device, sendmesh: &Sender<(i32, i32, u32, HashMap<u32, Arc<Chunk>>)>, origin: Vector3<f32>) {
        let mut slices = (-(self.render_distance as i32)..=(self.render_distance as i32)).flat_map(|x| {
            (-(self.render_distance as i32)..=(self.render_distance as i32)).flat_map(move |z| {
                (0..16).map(move |y| {
                    Vector3::new(x as f32, y as f32, z as f32)
                })
            })
        }).collect::<Vec<_>>();

        slices.sort_by(|a, b| {a.distance(origin).partial_cmp(&b.distance(origin)).unwrap()});

        for slice in slices {
            sendmesh.send((slice.x as i32, slice.z as i32, slice.y as u32, self.chunks.clone())).unwrap();
        }
    }

    pub fn mesh_slice(&self, device: &wgpu::Device, chunk: &Chunk, y_slice: u32) -> ((wgpu::Buffer, wgpu::Buffer, u32), (wgpu::Buffer, wgpu::Buffer, u32, Vec<Quad>)) {
        let ((vertices, indices, _), (vertices_transparent, indices_transparent, _, quads)) = mesh_slice_arrayed(chunk.position.x, chunk.position.y, y_slice, &self.chunks);

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

        ((vertex_buffer, index_buffer, ilen), ((vertex_buffer_t, index_buffer_t, ilen_t, quads)))
    }
    #[inline]
    pub fn get_block_at_absolute(&self, x: i32, y: i32, z: i32) -> Option<&BlockType> {
        if y < 0 || y > 255 {return None};
        let chunk_x = x.div_euclid(16);
        let chunk_z = z.div_euclid(16);

        self.chunks.get(&xz_to_index(chunk_x, chunk_z)).map(|v| v.get_block_at(x.rem_euclid(16) as u32, y as u32, z.rem_euclid(16) as u32))
    }

    /**
     * MUST BE DONE ON MAIN THREAD
     */
    pub fn break_block(&mut self, device: &wgpu::Device, x: i32, y: u32, z: i32) {
        let index = xz_to_index(x.div_euclid(16), z.div_euclid(16));
        let chunk = Arc::make_mut(self.chunks.get_mut(&index).unwrap());

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
        let yd = y.div_euclid(16) as i32;

        let mut requires_meshing = self.flood_lights_from_broken(Vector3::new(x, y as i32, z));

        let additional = vec![
            Vector3::new(xd + 1, yd, zd),
            Vector3::new(xd - 1, yd, zd),
            Vector3::new(xd, yd + 1, zd),
            Vector3::new(xd, yd - 1, zd),
            Vector3::new(xd, yd, zd + 1),
            Vector3::new(xd, yd, zd - 1),
            Vector3::new(xd, yd, zd)
        ];

        for v in additional {
            requires_meshing.insert(v);
        }

        let xyz: Vector3<f32> = Vector3::new(
            if xrem == 0 {xd - 1} else if xrem == 15 {xd + 1} else {xd} as f32,
            if yrem == 0 {yd - 1} else if yrem == 15 {yd + 1} else {yd} as f32,
            if zrem == 0 {zd - 1} else if zrem == 15 {zd + 1} else {zd} as f32,
        );

        requires_meshing.iter().collect::<Vec<&Vector3<i32>>>().sort_by(|a, b| (a.map(|v| v as f32) - xyz).magnitude().partial_cmp(&(b.map(|v| v as f32) - xyz).magnitude()).unwrap());

        for v in requires_meshing {
            self.update_queue.update_chunk_mesh(Vector3::new(v.x as i32, v.y as i32, v.z as i32));
        }
    }
    /**
     * MUST BE DONE ON MAIN THREAD
     */
    pub fn place_block(&mut self, device: &wgpu::Device, mut block: BlockType) {

        let abs = block.get_absolute_position();

        let index = xz_to_index(abs.x.div_euclid(16), abs.z.div_euclid(16));

        let local = block.get_relative_position();

        let chunk = Arc::make_mut(self.chunks.get_mut(&index).unwrap());

        let prevsun = chunk.grid[(abs.y / 16) as usize][local_xyz_to_index(local.x, local.y, local.z) as usize].get_sunlight_intensity();

        chunk.grid[(abs.y / 16) as usize][local_xyz_to_index(local.x, local.y, local.z) as usize] = block;

        let xd = abs.x.div_euclid(16);
        let zd = abs.z.div_euclid(16);
        let yd = abs.y.div_euclid(16);

        let mut requires_meshing = self.flood_lights_from_placed(abs, prevsun);

        let additional = vec![
            Vector3::new(xd + 1, yd, zd),
            Vector3::new(xd - 1, yd, zd),
            Vector3::new(xd, yd + 1, zd),
            Vector3::new(xd, yd - 1, zd),
            Vector3::new(xd, yd, zd + 1),
            Vector3::new(xd, yd, zd - 1),
            Vector3::new(xd, yd, zd)
        ];

        for v in additional {
            requires_meshing.insert(v);
        }

        let xyz: Vector3<f32> = Vector3::new(
            if local.x == 0 {xd - 1} else if local.x == 15 {xd + 1} else {xd} as f32,
            if local.y == 0 {yd - 1} else if local.y == 15 {yd + 1} else {yd} as f32,
            if local.z == 0 {zd - 1} else if local.z == 15 {zd + 1} else {zd} as f32,
        );

        requires_meshing.iter().collect::<Vec<&Vector3<i32>>>().sort_by(|a, b| (a.map(|v| v as f32) - xyz).magnitude().partial_cmp(&(b.map(|v| v as f32) - xyz).magnitude()).unwrap());

        for v in requires_meshing {
            self.update_queue.update_chunk_mesh(Vector3::new(v.x as i32, v.y as i32, v.z as i32));
        }
    }

    pub fn modify_block_at<F>(x: i32, y: u32, z: i32, chunks: &mut HashMap<u32, Arc<Chunk>>, callback: F) where F: FnMut(&mut BlockType) {
        if y > 255 {return};

        let cx = x.div_euclid(16);
        let cz = z.div_euclid(16);

        let xmod = x.rem_euclid(16) as u32;
        let zmod = z.rem_euclid(16) as u32;

        let xz = xz_to_index(cx, cz);

        let chunk_raw = chunks.get(&xz);

        if let Some(chunk) = chunk_raw {
            let mut write = Arc::make_mut(chunks.get_mut(&xz).unwrap());
            write.modify_block_at(xmod, y, zmod, callback);
        }
    }

    pub fn get_sunlight_intensity_at(x: i32, y: u32, z: i32, chunks: &HashMap<u32, Arc<Chunk>>) -> u8 {
        if y > 255 {return 0};

        let cx = x.div_euclid(16);
        let cz = z.div_euclid(16);

        let xmod = x.rem_euclid(16) as u32;
        let zmod = z.rem_euclid(16) as u32;

        let xz = xz_to_index(cx, cz);

        let chunk_raw = chunks.get(&xz);

        if let Some(chunk) = chunk_raw {
            return chunk.get_block_at(xmod, y, zmod).get_sunlight_intensity();
        }
        return 0;
    }

    pub fn set_sunlight_intensity_at(x: i32, y: u32, z: i32, chunks: &mut HashMap<u32, Arc<Chunk>>, val: u8) {
        if y > 255 {return};

        let cx = x.div_euclid(16);
        let cz = z.div_euclid(16);

        let xmod = x.rem_euclid(16) as u32;
        let zmod = z.rem_euclid(16) as u32;

        let xz = xz_to_index(cx, cz);

        let chunk_raw = chunks.get(&xz);

        if let Some(chunk) = chunk_raw {
            let mut write = Arc::make_mut(chunks.get_mut(&xz).unwrap());
            write.modify_block_at(xmod, y, zmod, |v| {
                v.set_sunlight_intensity(val);
            });
        }
    }

    pub fn flood_lights_from_placed(&mut self, pos: Vector3<i32>, prevsun: u8) -> HashSet<Vector3<i32>> {
        let mut set = HashSet::new();
        let mut queue: VecDeque<LightingBFSRemoveNode> = VecDeque::new();
        let mut prop_queue: VecDeque<LightingBFSAddNode> = VecDeque::new();
        set.insert(Vector3::new(pos.x.div_euclid(16), pos.y.div_euclid(16), pos.z.div_euclid(16)));
        
        ChunkManager::set_sunlight_intensity_at(pos.x, pos.y as u32, pos.z, &mut self.chunks, 0);
        queue.push_back(LightingBFSRemoveNode {
            position: pos,
            intensity: prevsun
        });

        while queue.len() > 0 {
            let item = queue.pop_front().unwrap();
            let pos = item.position;
            let intensity = item.intensity;

            let adj = [
                Vector3::new(pos.x + 1, pos.y, pos.z),
                Vector3::new(pos.x - 1, pos.y, pos.z),
                Vector3::new(pos.x, pos.y, pos.z + 1),
                Vector3::new(pos.x, pos.y, pos.z - 1),
                Vector3::new(pos.x, pos.y + 1, pos.z),
                Vector3::new(pos.x, pos.y - 1, pos.z),
            ];

            adj.map(|r| {
                let v = get_block_at_absolute(r.x, r.y, r.z, &self.chunks);
                if let Some(x) = v {
                    let pos2 = x.get_absolute_position();
                    let i = ChunkManager::get_sunlight_intensity_at(pos2.x, pos2.y as u32, pos2.z, &self.chunks);
                    drop(x);
                    if (i < intensity && i != 0) || (intensity == 15 && pos2.y == pos.y - 1) {
                        ChunkManager::set_sunlight_intensity_at(pos2.x, pos2.y as u32, pos2.z, &mut self.chunks, 0);
                        queue.push_back(LightingBFSRemoveNode {
                            position: pos2,
                            intensity: i
                        });
                        set.insert(Vector3::new(pos2.x.div_euclid(16), pos2.y.div_euclid(16), pos2.z.div_euclid(16)));
                    }
                    else if i >= intensity {
                        prop_queue.push_back(LightingBFSAddNode {
                            position: pos2
                        });
                    }
                }
            });
        }

        while prop_queue.len() > 0 {
            let item = prop_queue.pop_front().unwrap();
            let pos = item.position;
            let block = get_block_at_absolute(pos.x, pos.y, pos.z, &self.chunks).unwrap();
            let intensity = ChunkManager::get_sunlight_intensity_at(pos.x, pos.y as u32, pos.z, &self.chunks);
            drop(block);
            [
                Vector3::new(pos.x + 1, pos.y, pos.z),
                Vector3::new(pos.x - 1, pos.y, pos.z),
                Vector3::new(pos.x, pos.y, pos.z + 1),
                Vector3::new(pos.x, pos.y, pos.z - 1),
                Vector3::new(pos.x, pos.y + 1, pos.z),
                Vector3::new(pos.x, pos.y - 1, pos.z),
            ].map(|r| {
                let v = get_block_at_absolute(r.x, r.y, r.z, &self.chunks);
                if let Some(x) = v {
                    let xp = x.get_absolute_position();
                    if ChunkManager::get_sunlight_intensity_at(xp.x, xp.y as u32, xp.z, &self.chunks) + 2 <= intensity && x.has_partial_transparency() {
                        
                        drop(x);
                        if intensity == 15 && xp.y == pos.y - 1 {
                            ChunkManager::set_sunlight_intensity_at(xp.x, xp.y as u32, xp.z, &mut self.chunks, 15);
                        }
                        else {
                            ChunkManager::set_sunlight_intensity_at(xp.x, xp.y as u32, xp.z, &mut self.chunks, intensity - 1);
                        }
                        set.insert(Vector3::new(xp.x.div_euclid(16), xp.y.div_euclid(16), xp.z.div_euclid(16)));
                        prop_queue.push_back(LightingBFSAddNode {
                            position: xp
                        });
                    }
                }
            });
        }

        set
    }

    pub fn flood_lights_from_broken(&mut self, pos: Vector3<i32>) -> HashSet<Vector3<i32>> {
        let mut queue: VecDeque<LightingBFSAddNode> = VecDeque::new();
        let mut set = HashSet::new();
        let max_intensity_around = [
            Vector3::new(pos.x + 1, pos.y, pos.z),
            Vector3::new(pos.x - 1, pos.y, pos.z),
            Vector3::new(pos.x, pos.y, pos.z + 1),
            Vector3::new(pos.x, pos.y, pos.z - 1),
            Vector3::new(pos.x, pos.y + 1, pos.z),
            Vector3::new(pos.x, pos.y - 1, pos.z),
        ].into_iter().filter_map(|pos| Some(ChunkManager::get_sunlight_intensity_at(pos.x, pos.y as u32, pos.z, &self.chunks))).max().unwrap();
        
        let gi = ChunkManager::get_sunlight_intensity_at(pos.x, pos.y as u32 + 1, pos.z, &self.chunks);

        ChunkManager::set_sunlight_intensity_at(pos.x, pos.y as u32, pos.z, &mut self.chunks, if max_intensity_around == 0 {0} 
            else {
                if gi == max_intensity_around {max_intensity_around} 
                else {max_intensity_around - 1}
            }
        );

        queue.push_back(LightingBFSAddNode {
            position: pos
        });
        set.insert(Vector3::new(pos.x.div_euclid(16), pos.y.div_euclid(16), pos.z.div_euclid(16)));

        while queue.len() > 0 {
            let item = queue.pop_front().unwrap();
            let pos = item.position;
            let block = get_block_at_absolute(pos.x, pos.y, pos.z, &self.chunks).unwrap();
            let intensity = ChunkManager::get_sunlight_intensity_at(pos.x, pos.y as u32, pos.z, &self.chunks);
            drop(block);
            [
                Vector3::new(pos.x + 1, pos.y, pos.z),
                Vector3::new(pos.x - 1, pos.y, pos.z),
                Vector3::new(pos.x, pos.y, pos.z + 1),
                Vector3::new(pos.x, pos.y, pos.z - 1),
                Vector3::new(pos.x, pos.y + 1, pos.z),
                Vector3::new(pos.x, pos.y - 1, pos.z),
            ].map(|r| {
                let v = get_block_at_absolute(r.x, r.y, r.z, &self.chunks);
                if let Some(x) = v {
                    let xp = x.get_absolute_position();
                    
                    if ChunkManager::get_sunlight_intensity_at(xp.x, xp.y as u32, xp.z, &self.chunks) + 2 <= intensity && x.has_partial_transparency() {
                        
                        drop(x);
                        if intensity == 15 && xp.y == pos.y - 1 {
                            ChunkManager::set_sunlight_intensity_at(xp.x, xp.y as u32, xp.z, &mut self.chunks, 15);
                        }
                        else {
                            ChunkManager::set_sunlight_intensity_at(xp.x, xp.y as u32, xp.z, &mut self.chunks, intensity - 1);
                        }
                        set.insert(Vector3::new(xp.x.div_euclid(16), xp.y.div_euclid(16), xp.z.div_euclid(16)));
                        queue.push_back(LightingBFSAddNode {
                            position: xp
                        });
                    }
                }
            });
        }
        set
    }

    pub fn flood_lights(&mut self, chunk_index: u32) {
        let c = self.chunks.get(&chunk_index).unwrap();
        
        let ax = c.position.x;
        let az = c.position.y;
        
        drop(c);
        
        for x in 0..16 {
            for z in 0..16 {
                for y in (0..=255).rev() {

                    //set sunlight intensity of all transparent blocks above the surface to be 15

                    ChunkManager::set_sunlight_intensity_at(x + ax * 16, y as u32, z + az * 16, &mut self.chunks, 15);

                    let block = get_block_at_absolute(x + ax * 16, y, z + az * 16, &self.chunks).unwrap();
                    
                    let block_below = get_block_at_absolute(x + ax * 16, y - 1, z + az * 16, &self.chunks);
                    
                    
                    //if it is the first solid block hit(for light)...
                    if block.has_partial_transparency() && block_below.is_some() {
                        let bu = block_below.unwrap();
                        if bu.has_partial_transparency() {continue}
                        //start spreading light...
                        let mut queue: VecDeque<LightingBFSAddNode>  = VecDeque::new();
                        drop(block);
                        
                        //we'll get the updated block
                        let block = get_block_at_absolute(x + ax * 16, y, z + az * 16, &self.chunks).unwrap();
                        queue.push_back(LightingBFSAddNode {
                            position: block.get_absolute_position()
                        });

                        drop(block);
                        drop(bu);

                        while queue.len() > 0 {
                            let item = queue.pop_front().unwrap();
                            let pos = item.position;
                            let block = get_block_at_absolute(pos.x, pos.y, pos.z, &self.chunks).unwrap();
                            let intensity = ChunkManager::get_sunlight_intensity_at(pos.x, pos.y as u32, pos.z, &self.chunks);
                            
                            drop(block);
                            [
                                Vector3::new(pos.x + 1, pos.y, pos.z),
                                Vector3::new(pos.x - 1, pos.y, pos.z),
                                Vector3::new(pos.x, pos.y, pos.z + 1),
                                Vector3::new(pos.x, pos.y, pos.z - 1),
                                Vector3::new(pos.x, pos.y + 1, pos.z),
                                Vector3::new(pos.x, pos.y - 1, pos.z),
                            ].map(|r| {
                                let v = get_block_at_absolute(r.x, r.y, r.z, &self.chunks);
                                if let Some(x) = v {
                                    let xp = x.get_absolute_position();
                                    if ChunkManager::get_sunlight_intensity_at(xp.x, xp.y as u32, xp.z, &self.chunks) + 2 <= intensity && x.has_partial_transparency() {
                                        
                                        drop(x);
                                        if intensity == 15 && xp.y == pos.y - 1 {
                                            ChunkManager::set_sunlight_intensity_at(xp.x, xp.y as u32, xp.z, &mut self.chunks, 15);
                                        }
                                        else {
                                            ChunkManager::set_sunlight_intensity_at(xp.x, xp.y as u32, xp.z, &mut self.chunks, intensity - 1);
                                        }
                                        queue.push_back(LightingBFSAddNode {
                                            position: xp
                                        });
                                    }
                                }
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

    pub fn finalize_mesh(&mut self, x: i32, z: i32, slice: u32, device: &wgpu::Device, data: ((Vec<SurfaceVertex>, Vec<u32>, u32), (Vec<SurfaceVertex>, Vec<u32>, u32, Vec<Quad>))) {
        let ((vertices, indices, ilen), (vertices_transparent, indices_transparent, ilen_t, quads)) = (data.0, data.1);

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

        let chunk = self.chunk_buffers.get_mut(&xz_to_index(x, z)).unwrap();

        chunk.set_solid_buffer(slice, (vertex_buffer, index_buffer, ilen));
        
        chunk.set_transparent_buffer(slice, (vertex_buffer_t, index_buffer_t, ilen_t));

        let actual_chunk = Arc::make_mut(self.chunks.get_mut(&xz_to_index(x, z)).unwrap());
        actual_chunk.states[slice as usize] = ChunkState::Ready;
        actual_chunk.transparent_quads[slice as usize] = quads;

    }

    
}