use cgmath::Vector3;
use core::fmt::Debug;

pub type BlockType = Box<dyn Block + Send + Sync>;

pub enum BlockFace {
    Top = 0,
    Bottom = 1,
    Right = 2,
    Left = 3,
    Front = 4,
    Back = 5,
}

pub trait Block {
    /**
     the block's position in the world
    */
    fn get_absolute_position(&self) -> Vector3<i32>;
    /**
     the block's position in the (sub)chunk
    */
    fn get_relative_position(&self) -> Vector3<u32>;

    fn has_partial_transparency(&self) -> bool;
    fn does_mesh(&self) -> bool;

    fn get_name(&self) -> String;
    fn is_fluid(&self) -> bool;

    fn get_surface_textures(&self, face: BlockFace) -> (usize, usize, usize);

    // alright, here's how lighting will be done.
    // a 4 byte uint will be used to store light values
    // 8 bits r, 8 bits g, 8 bits b, 4 bits rgb intensity, 4 bits sun intensity
    // this data will be stored in the vertex and will require the subchunk it belongs to to be remeshed on change(added to a queue)
    
    fn reset_light(&mut self);
    fn set_sunlight_intensity(&mut self, intensity: u8);
    fn set_light(&mut self, with_color: [u8; 3]);
    fn get_light(&self) -> [u8; 3];
    fn get_sunlight_intensity(&self) -> u8;
}

impl Debug for dyn Block {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", format!("Block {:?}", self.get_name()))
    }
}

impl Debug for dyn Block + Send {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", format!("Block {:?}", self.get_name()))
    }
}

impl Debug for dyn Block + Send + Sync {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", format!("Block {:?}", self.get_name()))
    }
}