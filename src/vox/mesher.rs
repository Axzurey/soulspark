use std::{collections::HashMap, sync::{Arc, RwLock}};

use stopwatch::Stopwatch;
use wgpu::util::DeviceExt;

use crate::{blocks::block::{BlockFace, Blocks}, engine::surfacevertex::SurfaceVertex};

use super::{chunk::Chunk, chunk_manager::get_block_at_absolute_cloned};

const CHUNK_SIZE: usize = 16;

//based on https://gist.github.com/Vercidium/a3002bd083cce2bc854c9ff8f0118d33
pub fn mesh_slice(device: &wgpu::Device, chunk: &Chunk, y_slice: u32, chunks: &HashMap<u32, Arc<RwLock<Chunk>>>) -> ((wgpu::Buffer, wgpu::Buffer, u32), (wgpu::Buffer, wgpu::Buffer, u32)) {
    let t = Stopwatch::start_new();
    let mut vertices: Vec<SurfaceVertex> = Vec::with_capacity(16 * 16 * 16 * 6 * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(16 * 16 * 16 * 6 * 6);

    let mut vertices_transparent: Vec<SurfaceVertex> = Vec::with_capacity(16 * 16 * 16 * 6 * 4);
    let mut indices_transparent: Vec<u32> = Vec::with_capacity(16 * 16 * 16 * 6 * 6);

    let (chunk_pos_x, chunk_pos_y, chunk_pos_z) = (chunk.position.x, y_slice as i32, chunk.position.y);
    for d in 0..3 {
        let mut x = [0; 3];
        let mut q = [0; 3];
        let mut mask = vec![false; CHUNK_SIZE * CHUNK_SIZE];
        q[d] = 1;

        x[d] = -1;
        while x[d] < CHUNK_SIZE as i32 {
            let mut n = 0;
            for v in 0..CHUNK_SIZE {
                for u in 0..CHUNK_SIZE {
                    x[(d + 1) % 3] = u as i32;
                    x[(d + 2) % 3] = v as i32;

                    let block_current = get_block_at_absolute_cloned(x[0] + chunk_pos_x, x[1] + chunk_pos_y, x[2] + chunk_pos_z, &chunks);
                    let block_compare = get_block_at_absolute_cloned(
                        x[0] + q[0] + chunk_pos_x,
                        x[1] + q[1] + chunk_pos_y,
                        x[2] + q[2] + chunk_pos_z,
                        &chunks
                    );

                    let mut visible_face = false;

                    if block_current.is_none() {
                        mask[n] = visible_face;
                        n += 1;
                        continue;
                    }

                    if !block_current.unwrap().has_partial_transparency() {
                        if block_compare.is_none() || block_compare.unwrap().get_block() == Blocks::AIR {
                            visible_face = true;
                        }
                    }

                    mask[n] = visible_face;
                    n += 1;
                }
            }

            x[d] += 1;

            n = 0;
            for j in 0..CHUNK_SIZE {
                let mut i = 0;
                while i < CHUNK_SIZE {
                    if mask[n] {
                        let mut w = 1;
                        while i + w < CHUNK_SIZE && mask[n + w] {
                            w += 1;
                        }

                        let mut h = 1;
                        let mut done = false;
                        while j + h < CHUNK_SIZE {
                            for k in 0..w {
                                if !mask[n + k + h * CHUNK_SIZE] {
                                    done = true;
                                    break;
                                }
                            }
                            if done {
                                break;
                            }
                            h += 1;
                        }

                        x[(d + 1) % 3] = i as i32;
                        x[(d + 2) % 3] = j as i32;

                        let mut du = [0; 3];
                        let mut dv = [0; 3];
                        du[(d + 1) % 3] = w as i32;
                        dv[(d + 2) % 3] = h as i32;

                        let new_vertices = [
                            (x[0], x[1], x[2]),
                            (x[0] + du[0], x[1] + du[1], x[2] + du[2]),
                            (x[0] + dv[0], x[1] + dv[1], x[2] + dv[2]),
                            (x[0] + du[0] + dv[0], x[1] + du[1] + dv[1], x[2] + du[2] + dv[2])
                        ].map(|pos| {
                            SurfaceVertex::from_position(
                                [pos.0 as u32, pos.1 as u32, pos.2 as u32], BlockFace::Top, j as u32, (1, 0, 0), 15
                            )
                        });

                        indices.extend(vec![0, 1, 2, 1, 3, 2].iter());
                        vertices.extend(new_vertices);

                        for l in 0..h {
                            for k in 0..w {
                                mask[n + k + l * CHUNK_SIZE] = false;
                            }
                        }

                        i += w;
                        n += w;
                    } else {
                        i += 1;
                        n += 1;
                    }
                }
            }
        }
    }

    println!("Meshing prior {}micros", t.elapsed().as_micros());

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

    println!("Meshing {}micros", t.elapsed().as_micros());

    ((vertex_buffer, index_buffer, ilen), ((vertex_buffer_t, index_buffer_t, ilen_t)))
}