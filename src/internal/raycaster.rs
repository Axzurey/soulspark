use cgmath::Vector3;

use crate::{blocks::{airblock::AirBlock, block::{Block, BlockType, Blocks}}, vox::chunk_manager::{get_block_at_absolute, ChunkManager}};

pub struct BlockRaycastResult {
    pub hit: BlockType,
    pub normal: Vector3<i32>,
    pub position: Vector3<f32>
}

pub fn raycast_blocks<I>(from: Vector3<f32>, direction: Vector3<f32>, distance: f32, chunk_manager: &ChunkManager, ignore: I) -> Option<BlockRaycastResult>
    where I: Fn(&BlockType) -> bool
{
    //based on http://www.cse.yorku.ca/~amana/research/grid.pdf + https://github.com/fenomas/fast-voxel-raycast/blob/master/index.js
    let mut traversed: f32 = 0.;

    let mut ix = from.x.floor() as i32;
    let mut iy = from.y.floor() as i32;
    let mut iz = from.z.floor() as i32; 

    let step_x = if direction.x.is_sign_positive() { 1 } else { -1 };
    let step_y = if direction.y.is_sign_positive() { 1 } else { -1 };
    let step_z = if direction.z.is_sign_positive() { 1 } else { -1 };

    let delta_x = if direction.x != 0. {(1. / direction.x).abs()} else {f32::MAX};
    let delta_y = if direction.y != 0. {(1. / direction.y).abs()} else {f32::MAX};
    let delta_z = if direction.z != 0. {(1. / direction.z).abs()} else {f32::MAX};

    let dst_x = if step_x > 0 { ix as f32 + 1. - from.x } else { from.x - ix as f32 };
    let dst_y = if step_y > 0 { iy as f32 + 1. - from.y } else { from.y - iy as f32 };
    let dst_z = if step_z > 0 { iz as f32 + 1. - from.z } else { from.z - iz as f32 };

    let mut t_max_x = if direction.x != 0. { delta_x * dst_x } else { f32::MAX };
    let mut t_max_y = if direction.y != 0. { delta_y * dst_y } else { f32::MAX };
    let mut t_max_z = if direction.z != 0. { delta_z * dst_z } else { f32::MAX };

    while traversed < distance {
        let stepped_index;

        if t_max_x < t_max_y && t_max_x < t_max_z {
            ix += step_x;
            traversed = t_max_x;
            t_max_x += delta_x;
            stepped_index = 0;
        }
        else if t_max_y < t_max_z {
            iy += step_y;
            traversed = t_max_y;
            t_max_y += delta_y;
            stepped_index = 1;
        }
        else {
            iz += step_z;
            traversed = t_max_z;
            t_max_z += delta_z;
            stepped_index = 2;
        }

        let block = get_block_at_absolute(ix, iy, iz, &chunk_manager.chunks);

        match block {
            Some(b) => {
                if b.get_block() != Blocks::AIR {
                    if !ignore(&b) {
                        //println!("{:?}, \n{:?}", from + direction * traversed, Vector3::new(x, y, z));
                        return Some(BlockRaycastResult {
                            hit: b.as_ref().clone(),
                            position: from + direction * traversed,
                            normal: Vector3::new(
                                if stepped_index == 0 {-step_x} else {0},
                                if stepped_index == 1 {-step_y} else {0},
                                if stepped_index == 2 {-step_z} else {0},
                            )
                        });
                    }
                }
            }
            None => {}, 
        };
    }
    None
}