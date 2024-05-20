use cgmath::Vector3;

use crate::engine::texture_loader::get_indices_from_texture;

use super::block::{Block, BlockFace};

pub struct GrassBlock {
    relative_position: Vector3<u32>,
    absolute_position: Vector3<i32>,
    sunlight_intensity: u8,
    lights: [u8; 3]
}

impl GrassBlock {
    pub fn new(
        relative_position: Vector3<u32>,
        absolute_position: Vector3<i32>
    ) -> Self {
        Self {
            relative_position,
            absolute_position
        }
    }
}

impl Block for GrassBlock {
    fn get_absolute_position(&self) -> Vector3<i32> {
        self.absolute_position
    }

    fn get_relative_position(&self) -> Vector3<u32> {
        self.relative_position
    }

    fn has_partial_transparency(&self) -> bool {
        false
    }

    fn does_mesh(&self) -> bool {
        true
    }

    fn get_name(&self) -> String {
        "grass block".to_owned()
    }

    fn is_fluid(&self) -> bool {
        false
    }

    fn get_surface_textures(&self, face: super::block::BlockFace) -> (usize, usize, usize) {
        
        (match face {
            BlockFace::Top => {
                get_indices_from_texture("grass-top")
            },
            BlockFace::Bottom => {
                get_indices_from_texture("dirt")
            },
            _ => {
                get_indices_from_texture("grass-side")
            }
        }, 0, 0)
    }
}