use std::{collections::{HashMap, VecDeque}, sync::{Arc, RwLock}};

use glam::{ivec2, ivec3, IVec2, IVec3, Vec3};
use stopwatch::Stopwatch;
use wgpu::util::DeviceExt;

use crate::{blocks::block::{BlockFace, BlockType, Blocks}, engine::surfacevertex::SurfaceVertex};

use super::{chunk::Chunk, chunk_manager::get_block_at_absolute_cloned};

pub const ADJACENT_CHUNK_DIRECTIONS: [IVec3; 27] = [
    IVec3 { x: 0, y: 0, z: 0 },
    // moore neighbours in the negative direction
    IVec3 { x: 0, y: -1, z: -1 },
    IVec3 { x: -1, y: 0, z: -1 },
    IVec3 { x: -1, y: 0, z: 1 },
    IVec3 { x: -1, y: -1, z: 0 },
    IVec3 {
        x: -1,
        y: -1,
        z: -1,
    },
    IVec3 { x: -1, y: 1, z: -1 },
    IVec3 { x: -1, y: -1, z: 1 },
    IVec3 { x: -1, y: 1, z: 1 },
    IVec3 { x: 1, y: 0, z: -1 },
    IVec3 { x: 1, y: -1, z: -1 },
    IVec3 { x: 0, y: 1, z: -1 },
    IVec3 { x: 1, y: 1, z: 1 },
    IVec3 { x: 1, y: -1, z: 1 },
    IVec3 { x: 1, y: 1, z: -1 },
    IVec3 { x: 1, y: 1, z: 0 },
    IVec3 { x: 0, y: 1, z: 1 },
    IVec3 { x: 1, y: -1, z: 0 },
    IVec3 { x: 0, y: -1, z: 1 },
    IVec3 { x: 1, y: 0, z: 1 },
    IVec3 { x: -1, y: 1, z: 0 },
    // von neumann neighbour
    IVec3 { x: -1, y: 0, z: 0 },
    IVec3 { x: 1, y: 0, z: 0 },
    IVec3 { x: 0, y: -1, z: 0 },
    IVec3 { x: 0, y: 1, z: 0 },
    IVec3 { x: 0, y: 0, z: -1 },
    IVec3 { x: 0, y: 0, z: 1 },
];

pub const ADJACENT_AO_DIRS: [IVec2; 9] = [
    ivec2(-1, -1),
    ivec2(-1, 0),
    ivec2(-1, 1),
    ivec2(0, -1),
    ivec2(0, 0),
    ivec2(0, 1),
    ivec2(1, -1),
    ivec2(1, 0),
    ivec2(1, 1),
];

#[inline]
pub fn index_to_ivec3(i: i32) -> IVec3 {
    let x = i % 32;
    let y = (i / 32) % 32;
    let z = i / (32 * 32);
    IVec3::new(x, y, z)
}

#[inline]
pub fn index_to_ivec3_bounds(i: i32, bounds: i32) -> IVec3 {
    let x = i % bounds;
    let y = (i / bounds) % bounds;
    let z = i / (bounds * bounds);
    IVec3::new(x, y, z)
}

#[inline]
pub fn index_to_ivec3_bounds_reverse(i: i32, bounds: i32) -> IVec3 {
    let z = i % bounds;
    let y = (i / bounds) % bounds;
    let x = i / (bounds * bounds);
    IVec3::new(x, y, z)
}

#[inline]
pub fn is_on_edge(pos: IVec3) -> bool {
    if pos.x == 0 || pos.x == 32 {
        return true;
    }
    if pos.y == 0 || pos.y == 32 {
        return true;
    }
    if pos.z == 0 || pos.z == 32 {
        return true;
    }
    false
}

///! if lying on the edge of our chunk, return the edging chunk
#[inline]
pub fn get_edging_chunk(pos: IVec3) -> Option<IVec3> {
    let mut chunk_dir = IVec3::ZERO;
    if pos.x == 0 {
        chunk_dir.x = -1;
    } else if pos.x == 31 {
        chunk_dir.x = 1;
    }
    if pos.y == 0 {
        chunk_dir.y = -1;
    } else if pos.y == 31 {
        chunk_dir.y = 1;
    }
    if pos.z == 0 {
        chunk_dir.z = -1;
    } else if pos.z == 31 {
        chunk_dir.z = 1;
    }
    if chunk_dir == IVec3::ZERO {
        None
    } else {
        Some(chunk_dir)
    }
}

// pos 18 bits, ao 3 bits, normal 4 bits
// 18-21-25-   left 32-25 = 7
#[inline]
pub fn make_vertex_u32(
    // position: [i32; 3], /*, normal: i32, color: Color, texture_id: u32*/
    pos: IVec3, /*, normal: i32, color: Color, texture_id: u32*/
    ao: u32,
    normal: u32,
    block_type: u32,
) -> u32 {
    pos.x as u32
        | (pos.y as u32) << 6u32
        | (pos.z as u32) << 12u32
        | ao << 18u32
        | normal << 21u32
        | block_type << 25u32
    // | (normal as u32) << 18u32
    // | (texture_id) << 21u32
}

#[inline]
pub fn world_to_chunk(pos: Vec3) -> IVec3 {
    ((pos - Vec3::splat(16.0)) * (1.0 / 32.0)).as_ivec3()
}

///! generate a vec of indices
///! assumes vertices are made of quads, and counter clockwise ordered
#[inline]
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

#[test]
fn index_functions() {
    for z in 0..32 {
        for y in 0..32 {
            for x in 0..32 {
                let pos = IVec3::new(x, y, z);
                let index = vec3_to_index(pos, 32);
                let from_index = index_to_ivec3_bounds(index as i32, 32);
                assert_eq!(pos, from_index);
            }
        }
    }
}

#[inline]
pub fn vec3_to_index(pos: IVec3, bounds: i32) -> usize {
    let x_i = pos.x % bounds;
    // let y_i = (pos.y * bounds) % bounds;
    let y_i = (pos.y * bounds);
    let z_i = pos.z * (bounds * bounds);
    // let x_i = pos.x % bounds;
    // let y_i = (pos.y / bounds) % bounds;
    // let z_i = pos.z / (bounds * bounds);
    (x_i + y_i + z_i) as usize
}

const CHUNK_SIZE: usize = 16;
const CHUNK_SIZE_P: usize = 18;
//https://github.com/TanTanDev/binary_greedy_mesher_demo/blob/main/src/greedy_mesher_optimized.rs
pub fn build_chunk_mesh(chunk: &Arc<RwLock<Chunk>>, chunks: &HashMap<u32, Arc<RwLock<Chunk>>>, slice: u32) -> Option<()> {

    // solid binary for each x,y,z axis (3)
    let mut axis_cols = [[[0u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3];

    // the cull mask to perform greedy slicing, based on solids on previous axis_cols
    let mut col_face_masks = [[[0u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6];

    #[inline]
    fn add_voxel_to_axis_cols(
        b: &BlockType,
        x: usize,
        y: usize,
        z: usize,
        axis_cols: &mut [[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3],
    ) {
        if !b.has_partial_transparency() {
            // x,z - y axis
            axis_cols[0][z][x] |= 1u64 << y as u64;
            // z,y - x axis
            axis_cols[1][y][z] |= 1u64 << x as u64;
            // x,y - z axis
            axis_cols[2][y][x] |= 1u64 << z as u64;
        }
    }

    // inner chunk voxels.
    let chunk = &chunks[&(vec3_to_index(IVec3::new(1, 1, 1), 3) as u32)].read().unwrap();
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let i = (z * CHUNK_SIZE + y) * CHUNK_SIZE + x;
                add_voxel_to_axis_cols(&chunk.get_block_at(x as u32, y as u32 + CHUNK_SIZE as u32 * slice, z as u32), x + 1, y + 1, z + 1, &mut axis_cols)
            }
        }
    }

    // neighbor chunk voxels.
    // note(leddoo): couldn't be bothered to optimize these.
    //  might be worth it though. together, they take
    //  almost as long as the entire "inner chunk" loop.
    for z in [0, CHUNK_SIZE_P - 1] {
        for y in 0..CHUNK_SIZE_P {
            for x in 0..CHUNK_SIZE_P {
                let pos = ivec3(x as i32, y as i32, z as i32) - IVec3::ONE;
                let block = get_block_at_absolute_cloned(x as i32 - 1, y as i32 - 1, z as i32 - 1, &chunks).unwrap();
                add_voxel_to_axis_cols(&block, x, y, z, &mut axis_cols);
            }
        }
    }
    for z in 0..CHUNK_SIZE_P {
        for y in [0, CHUNK_SIZE_P - 1] {
            for x in 0..CHUNK_SIZE_P {
                let pos = ivec3(x as i32, y as i32, z as i32) - IVec3::ONE;
                let block = get_block_at_absolute_cloned(x as i32 - 1, y as i32 - 1, z as i32 - 1, &chunks).unwrap();
                add_voxel_to_axis_cols(&block, x, y, z, &mut axis_cols);
            }
        }
    }
    for z in 0..CHUNK_SIZE_P {
        for x in [0, CHUNK_SIZE_P - 1] {
            for y in 0..CHUNK_SIZE_P {
                let block = get_block_at_absolute_cloned(x as i32 - 1, y as i32 - 1, z as i32 - 1, &chunks).unwrap();
                add_voxel_to_axis_cols(&block, x, y, z, &mut axis_cols);
            }
        }
    }

    // face culling
    for axis in 0..3 {
        for z in 0..CHUNK_SIZE_P {
            for x in 0..CHUNK_SIZE_P {
                // set if current is solid, and next is air
                let col = axis_cols[axis][z][x];

                // sample descending axis, and set true when air meets solid
                col_face_masks[2 * axis + 0][z][x] = col & !(col << 1);
                // sample ascending axis, and set true when air meets solid
                col_face_masks[2 * axis + 1][z][x] = col & !(col >> 1);
            }
        }
    }

    // greedy meshing planes for every axis (6)
    // key(block + ao) -> HashMap<axis(0-32), binary_plane>
    // note(leddoo): don't ask me how this isn't a massive blottleneck.
    //  might become an issue in the future, when there are more block types.
    //  consider using a single hashmap with key (axis, block_hash, y).
    let mut data: [HashMap<u32, HashMap<u32, [u32; 32]>>; 6];
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
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                // skip padded by adding 1(for x padding) and (z+1) for (z padding)
                let mut col = col_face_masks[axis][z + 1][x + 1];

                // removes the right most padding value, because it's invalid
                col >>= 1;
                // removes the left most padding value, because it's invalid
                col &= !(1 << CHUNK_SIZE as u64);

                while col != 0 {
                    let y = col.trailing_zeros();
                    // clear least significant set bit
                    col &= col - 1;

                    // get the voxel position based on axis
                    let voxel_pos = match axis {
                        0 | 1 => ivec3(x as i32, y as i32, z as i32), // down,up
                        2 | 3 => ivec3(y as i32, z as i32, x as i32), // left, right
                        _ => ivec3(x as i32, z as i32, y as i32),     // forward, back
                    };

                    // calculate ambient occlusion
                    let mut ao_index = 0;
                    for (ao_i, ao_offset) in ADJACENT_AO_DIRS.iter().enumerate() {
                        // ambient occlusion is sampled based on axis(ascent or descent)
                        let ao_sample_offset = match axis {
                            0 => ivec3(ao_offset.x, -1, ao_offset.y), // down
                            1 => ivec3(ao_offset.x, 1, ao_offset.y),  // up
                            2 => ivec3(-1, ao_offset.y, ao_offset.x), // left
                            3 => ivec3(1, ao_offset.y, ao_offset.x),  // right
                            4 => ivec3(ao_offset.x, ao_offset.y, -1), // forward
                            _ => ivec3(ao_offset.x, ao_offset.y, 1),  // back
                        };
                        let ao_voxel_pos = voxel_pos + ao_sample_offset;
                        let ao_block = get_block_at_absolute_cloned(
                            ao_voxel_pos.x,
                            ao_voxel_pos.y,
                            ao_voxel_pos.z,
                            &chunks
                        ).unwrap();
                        if !ao_block.has_partial_transparency() {
                            ao_index |= 1u32 << ao_i;
                        }
                    }

                    let current_voxel = chunk.get_block_at(voxel_pos.x as u32, voxel_pos.y as u32 + slice * 16, voxel_pos.z as u32);
                    // let current_voxel = chunks_refs.get_block(voxel_pos);
                    // we can only greedy mesh same block types + same ambient occlusion
                    let block_hash = ao_index | ((current_voxel.get_block() as u32) << 9);
                    let data = data[axis]
                        .entry(block_hash)
                        .or_default()
                        .entry(y)
                        .or_default();
                    data[x as usize] |= 1u32 << z as u32;
                }
            }
        }
    }

    let mut vertices = vec![];
    for (axis, block_ao_data) in data.into_iter().enumerate() {
        let facedir = match axis {
            0 => BlockFace::Bottom,
            1 => BlockFace::Top,
            2 => BlockFace::Left,
            3 => BlockFace::Right,
            4 => BlockFace::Front,
            _ => BlockFace::Back,
        };
        for (block_ao, axis_plane) in block_ao_data.into_iter() {
            let ao = block_ao & 0b111111111;
            let block_type = block_ao >> 9;
            for (axis_pos, plane) in axis_plane.into_iter() {
                let quads_from_axis = greedy_mesh_binary_plane(plane);

                quads_from_axis.into_iter().for_each(|q| {
                    q.append_vertices(&mut vertices, facedir, axis_pos, ao, block_type)
                });
            }
        }
    }

    mesh.vertices.extend(vertices);
    if mesh.vertices.is_empty() {
        None
    } else {
        mesh.indices = generate_indices(mesh.vertices.len());
        Some(mesh)
    }
}

// todo: compress further?
#[derive(Debug)]
pub struct GreedyQuad {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl GreedyQuad {
    ///! compress this quad data into the input vertices vec
    pub fn append_vertices(
        &self,
        vertices: &mut Vec<u32>,
        face_dir: BlockFace,
        axis: u32,
        ao: u32,
        block_type: u32,
    ) {
        // let negate_axis = face_dir.negate_axis();
        // let axis = axis as i32 + negate_axis;
        let axis = axis as i32;
        let jump = 1;

        // pack ambient occlusion strength into vertex
        let v1ao = ((ao >> 0) & 1) + ((ao >> 1) & 1) + ((ao >> 3) & 1);
        let v2ao = ((ao >> 3) & 1) + ((ao >> 6) & 1) + ((ao >> 7) & 1);
        let v3ao = ((ao >> 5) & 1) + ((ao >> 8) & 1) + ((ao >> 7) & 1);
        let v4ao = ((ao >> 1) & 1) + ((ao >> 2) & 1) + ((ao >> 5) & 1);

        let v1 = make_vertex_u32(
            face_dir.world_to_sample(axis as i32, self.x as i32, self.y as i32) * jump,
            v1ao,
            face_dir.normal_index(),
            block_type,
        );
        let v2 = make_vertex_u32(
            face_dir.world_to_sample(
                axis as i32,
                self.x as i32 + self.w as i32,
                self.y as i32,
            ) * jump,
            v2ao,
            face_dir.normal_index(),
            block_type,
        );
        let v3 = make_vertex_u32(
            face_dir.world_to_sample(
                axis as i32,
                self.x as i32 + self.w as i32,
                self.y as i32 + self.h as i32,
            ) * jump,
            v3ao,
            face_dir.normal_index(),
            block_type,
        );
        let v4 = make_vertex_u32(
            face_dir.world_to_sample(
                axis as i32,
                self.x as i32,
                self.y as i32 + self.h as i32,
            ) * jump,
            v4ao,
            face_dir.normal_index(),
            block_type,
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

        // anisotropy flip
        if (v1ao > 0) ^ (v3ao > 0) {
            // right shift array, to swap triangle intersection angle
            let f = new_vertices.pop_front().unwrap();
            new_vertices.push_back(f);
        }

        vertices.extend(new_vertices);
    }
}

///! generate quads of a binary slice
///! lod not implemented atm
pub fn greedy_mesh_binary_plane(mut data: [u32; 32]) -> Vec<GreedyQuad> {
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
            while row + w < 16 as usize {
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
                x: row as u32,
            });
            y += h;
        }
    }
    greedy_quads
}