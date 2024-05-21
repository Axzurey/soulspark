use cgmath::Vector3;

use super::block::Block;

pub struct AirBlock {
    relative_position: Vector3<u32>,
    absolute_position: Vector3<i32>,
    sunlight_intensity: u8,
    lights: [u8; 3]
}

impl AirBlock {
    pub fn new(
        relative_position: Vector3<u32>,
        absolute_position: Vector3<i32>
    ) -> Self {
        Self {
            relative_position,
            absolute_position,
            sunlight_intensity: 0,
            lights: [0, 0, 0]
        }
    }
}

impl Block for AirBlock {
    fn get_absolute_position(&self) -> Vector3<i32> {
        self.absolute_position
    }

    fn get_relative_position(&self) -> Vector3<u32> {
        self.relative_position
    }

    fn has_partial_transparency(&self) -> bool {
        true
    }

    fn does_mesh(&self) -> bool {
        false
    }

    fn get_name(&self) -> String {
        "dirt block".to_owned()
    }

    fn is_fluid(&self) -> bool {
        false
    }

    fn get_surface_textures(&self, face: super::block::BlockFace) -> (usize, usize, usize) {
        (0, 0, 0)
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
}