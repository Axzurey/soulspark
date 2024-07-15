use std::{collections::{HashMap, HashSet, VecDeque}, sync::Arc};

use cgmath::Vector3;
use parking_lot::RwLock;

use crate::{blocks::block::{calculate_illumination_bytes, BlockFace, BlockType, Blocks}, engine::surfacevertex::SurfaceVertex, internal::depthsort::Quad, vox::{chunk::xz_to_index, chunk_manager::get_block_at_absolute}};

use super::chunk::Chunk;

pub fn generate_indices(vertex_count: usize) -> Vec<u32> {
    let indices_count = vertex_count / 4;
    let mut indices = Vec::<u32>::with_capacity(indices_count);
    (0..indices_count).into_iter().for_each(|vert_index| {
        let vert_index = vert_index as u32 * 4u32;
        indices.push(vert_index);
        indices.push(vert_index + 1);
        indices.push(vert_index + 2);
        indices.push(vert_index);
        indices.push(vert_index + 2);
        indices.push(vert_index + 3);
    });
    indices
}

#[derive(PartialEq, Clone)]
pub enum MeshStageType {
    Solid,
    Transparent,
    Fluid
}

pub fn binary_mesh(chunk_x: i32, chunk_z: i32, y_slice: u32, chunks: &HashMap<u32, Arc<Chunk>>, stage: MeshStageType) -> (Vec<SurfaceVertex>, Vec<u32>, u32, Vec<Quad>) {
    let mut axis_columns = [[[0u32; 18]; 18]; 3];

    let mut column_face_masks = [[[0u32; 18]; 18]; 6];

    #[inline]
    fn add_voxel_to_axis_cols(
        is_transparent: bool,
        is_fluid: bool,
        x: usize,
        y: usize,
        z: usize,
        axis_cols: &mut [[[u32; 18]; 18]; 3],
        stage: &MeshStageType
    ) {
        if 
            (!is_transparent && *stage == MeshStageType::Solid) ||
            (is_transparent && *stage == MeshStageType::Transparent) ||
            (is_fluid && *stage == MeshStageType::Fluid)
          {
            // x,z - y axis
            axis_cols[0][z][x] |= 1u32 << y as u32;
            // z,y - x axis
            axis_cols[1][y][z] |= 1u32 << x as u32;
            // x,y - z axis
            axis_cols[2][y][x] |= 1u32 << z as u32;
        }
    }

    let chunk = chunks.get(&xz_to_index(chunk_x, chunk_z)).unwrap();

    for z in 0..16 {
        for y in 0..16 {
            for x in 0..16 {
                let b = chunk.get_block_at(x as u32, y as u32 + y_slice * 16, z as u32);
                if b.get_block() == Blocks::AIR {continue;}
                add_voxel_to_axis_cols(b.has_partial_transparency(), b.is_fluid(), x + 1, y + 1, z + 1, &mut axis_columns, &stage);
            }
        }
    }

    for z in [0, 18 - 1] {
        for y in 0..18 {
            for x in 0..18 {
                let pos = Vector3::new(x as i32 - 1, y as i32 - 1, z as i32 - 1);
                let block = get_block_at_absolute(pos.x + chunk_x * 16, pos.y + y_slice as i32 * 16, pos.z + chunk_z * 16, chunks);
                
                let hastrans = match &block {
                    Some(b) => b.has_partial_transparency(),
                    None => false
                };

                let isfluid = match &block {
                    Some(b) => b.is_fluid(),
                    None => false
                };

                if block.is_none() || block.unwrap().get_block() == Blocks::AIR {continue;}

                add_voxel_to_axis_cols(hastrans, isfluid, x, y, z, &mut axis_columns, &stage);
            }
        }
    }
    for z in 0..18 {
        for y in [0, 18 - 1] {
            for x in 0..18 {
                let pos = Vector3::new(x as i32 - 1, y as i32 - 1, z as i32 - 1);
                let block = get_block_at_absolute(pos.x + chunk_x * 16, pos.y + y_slice as i32 * 16, pos.z + chunk_z * 16, chunks);
                
                let hastrans = match &block {
                    Some(b) => b.has_partial_transparency(),
                    None => false
                };

                let isfluid = match &block {
                    Some(b) => b.is_fluid(),
                    None => false
                };

                if block.is_none() || block.unwrap().get_block() == Blocks::AIR {continue;}

                add_voxel_to_axis_cols(hastrans, isfluid, x, y, z, &mut axis_columns, &stage);
            }
        }
    }
    for z in 0..18 {
        for x in [0, 18 - 1] {
            for y in 0..18 {
                let pos = Vector3::new(x as i32 - 1, y as i32 - 1, z as i32 - 1);
                let block = get_block_at_absolute(pos.x + chunk_x * 16, pos.y + y_slice as i32 * 16, pos.z + chunk_z * 16, chunks);
                
                let hastrans = match &block {
                    Some(b) => b.has_partial_transparency(),
                    None => false
                };

                let isfluid = match &block {
                    Some(b) => b.is_fluid(),
                    None => false
                };

                if block.is_none() || block.unwrap().get_block() == Blocks::AIR {continue;}

                add_voxel_to_axis_cols(hastrans, isfluid, x, y, z, &mut axis_columns, &stage);
            }
        }
    }

    for axis in 0..3 {
        for z in 0..18 {
            for x in 0..18 {
                // set if current is solid, and next is air
                let col = axis_columns[axis][z][x];

                // sample descending axis, and set true when air meets solid
                column_face_masks[2 * axis + 0][z][x] = col & !(col << 1);
                // sample ascending axis, and set true when air meets solid
                column_face_masks[2 * axis + 1][z][x] = col & !(col >> 1);
            }
        }
    }

    let mut data: [HashMap<u64, HashMap<u32, ([u32; 16], Option<&BlockType>)>>; 6];
    data = [
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    ];

    // find faces and build binary planes based on the voxel block+ao etc...
    for axis in 0..6 {
        let facedir = match axis {
            0 => BlockFace::Bottom,
            1 => BlockFace::Top,
            2 => BlockFace::Left,
            3 => BlockFace::Right,
            4 => BlockFace::Front,
            _ => BlockFace::Back,
        };
        for z in 0..16 {
            for x in 0..16 {
                // skip padded by adding 1(for x padding) and (z+1) for (z padding)
                let mut col = column_face_masks[axis][z + 1][x + 1];

                col >>= 1;
                col &= !(1 << 16 as u32);

                while col != 0 {
                    let y = col.trailing_zeros();
                    col &= col - 1;

                    let voxel_pos = match axis {
                        0 | 1 => Vector3::new(x as i32, y as i32, z as i32),
                        2 | 3 => Vector3::new(y as i32, z as i32, x as i32),
                        _ => Vector3::new(x as i32, z as i32, y as i32),
                    };
                    

                    let current_voxel = chunk.get_block_at(voxel_pos.x as u32, voxel_pos.y as u32 + y_slice * 16, voxel_pos.z as u32);
                    
                    let nextdoorpos = current_voxel.get_absolute_position() + facedir.normal();

                    let illumination = get_block_at_absolute(nextdoorpos.x, nextdoorpos.y, nextdoorpos.z, chunks).map_or(0, |v| calculate_illumination_bytes(&v));

                    let block_hash = illumination as u64 | ((current_voxel.get_block() as u64) << 32);
                    let data = data[axis]
                        .entry(block_hash)
                        .or_default()
                        .entry(y)
                        .or_default();
                    data.0[x as usize] |= 1u32 << z as u32;
                    data.1 = Some(&current_voxel);
                }
            }
        }
    }

    let mut vertices = vec![];
    let mut quads: Vec<Quad> = vec![];

    for (axis, blockdata) in data.into_iter().enumerate() {
        let facedir = match axis {
            0 => BlockFace::Bottom,
            1 => BlockFace::Top,
            2 => BlockFace::Left,
            3 => BlockFace::Right,
            4 => BlockFace::Front,
            _ => BlockFace::Back,
        };
        for (_, axis_plane) in blockdata.into_iter() {
            for (axis_pos, plane) in axis_plane.into_iter() {
                let quads_from_axis = greedy_mesh_binary_plane(plane.0);

                quads_from_axis.into_iter().for_each(|q| {
                    q.append_vertices(&mut vertices, facedir, axis_pos, plane.1.unwrap(), chunks, &mut quads);
                });
            }
        }
    }

    let indices = generate_indices(vertices.len());
    let ilen = indices.len();


    (vertices, indices, ilen as u32, quads)
}

pub struct GreedyQuad {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl GreedyQuad {
    pub fn append_vertices(
        &self,
        vertices: &mut Vec<SurfaceVertex>,
        face_dir: BlockFace,
        axis: u32,
        block: &BlockType,
        chunks: &HashMap<u32, Arc<Chunk>>,
        quads: &mut Vec<Quad>
    ) {
        let axis = axis as i32;

        let nextdoorpos = block.get_absolute_position() + face_dir.normal();

        let illumination = get_block_at_absolute(nextdoorpos.x, nextdoorpos.y, nextdoorpos.z, chunks).map_or(0, |v| calculate_illumination_bytes(&v));

        let tex = block.get_surface_textures(face_dir);

        let v1 = SurfaceVertex::from_position(
            face_dir.world_to_sample(axis as i32, self.x as i32, self.y as i32), face_dir, 0, tex, illumination
        );
        let v2 = SurfaceVertex::from_position(
            face_dir.world_to_sample(axis as i32, self.x as i32 + self.w as i32, self.y as i32), face_dir, 1, tex, illumination
        );
        let v3 = SurfaceVertex::from_position(
            face_dir.world_to_sample(axis as i32, self.x as i32 + self.w as i32, self.y as i32 + self.h as i32), face_dir, 2, tex, illumination
        );
        let v4 = SurfaceVertex::from_position(
            face_dir.world_to_sample(axis as i32, self.x as i32, self.y as i32 + self.h as i32), face_dir, 3, tex, illumination
        );

        // the quad vertices to be added
        let mut new_vertices = VecDeque::from([v1, v2, v3, v4]);

        // triangle vertex order is different depending on the facing direction
        // due to indices always being the same
        if face_dir.reverse_order() {
            // keep first index, but reverse the rest
            let o = new_vertices.split_off(1);
            o.into_iter().rev().for_each(|i| new_vertices.push_back(i));
        }

        //todo: actually calculate the center
        let center = block.get_absolute_position().map(|v| v as f32);

        quads.push(Quad {
            center,
            vertices: [v1, v2, v3, v4]
        });
        
        vertices.extend(new_vertices);
    }
}

pub fn greedy_mesh_binary_plane(mut data: [u32; 16]) -> Vec<GreedyQuad> {
    let mut greedy_quads = vec![];
    for row in 0..data.len() {
        let mut y = 0;
        while y < 16 {
            // find first solid, "air/zero's" could be first so skip
            y += (data[row] >> y).trailing_zeros();
            if y >= 16 {
                // reached top
                continue;
            }
            let h = (data[row] >> y).trailing_ones();
            // convert height 'num' to positive bits repeated 'num' times aka:
            // 1 = 0b1, 2 = 0b11, 4 = 0b1111
            let h_as_mask = u32::checked_shl(1, h).map_or(!0, |v| v - 1);
            let mask = h_as_mask << y;
            // grow horizontally
            let mut w = 1;
            while row + w < 16 {
                // fetch bits spanning height, in the next row
                let next_row_h = (data[row + w] >> y) & h_as_mask;
                if next_row_h != h_as_mask {
                    break; // can no longer expand horizontally
                }

                // nuke the bits we expanded into
                data[row + w] = data[row + w] & !mask;

                w += 1;
            }
            greedy_quads.push(GreedyQuad {
                y,
                w: w as u32,
                h,
                x: row as u32
            });
            y += h;
        }
    }
    greedy_quads
}