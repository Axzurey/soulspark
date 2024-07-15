use cgmath::Vector3;

use crate::engine::texture_loader::get_indices_from_texture;

use super::block::Block;
#[derive(Clone)]
pub struct LogBlock {
    relative_position: Vector3<u32>,
    absolute_position: Vector3<i32>,
    sunlight_intensity: u8,
    lights: [u8; 3]
}

impl LogBlock {
    pub fn new(
        relative_position: Vector3<u32>,
        absolute_position: Vector3<i32>
    ) -> Self {
        Self {
            relative_position,
            absolute_position,
            lights: [0, 0, 0],
            sunlight_intensity: 0
        }
    }
}

impl Block for LogBlock {
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
        "log block".to_owned()
    }

    fn is_fluid(&self) -> bool {
        false
    }

    fn get_surface_textures(&self, face: super::block::BlockFace) -> (usize, usize, usize) {
        match face {
            super::block::BlockFace::Top | super::block::BlockFace::Bottom => {
                (get_indices_from_texture("log_top"), 0, 0)
            },
            _ => {
                (get_indices_from_texture("log"), 0, 0)
            }
        }
    }
    fn reset_light(&mut self) {
        self.lights = [0, 0, 0];
        self.sunlight_intensity = 0;
    }
    
    fn set_sunlight_intensity(&mut self, intensity: u8) {
        self.sunlight_intensity = intensity;
    }
    
    fn set_light(&mut self, with_color: [u8; 3]) {
        self.lights = with_color;
    }
    
    fn get_light(&self) -> &[u8; 3] {
        &self.lights
    }
    
    fn get_sunlight_intensity(&self) -> u8 {
        self.sunlight_intensity
    }
    fn get_block(&self) -> super::block::Blocks {
        super::block::Blocks::LOG
    }
}