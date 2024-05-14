use cgmath::Vector3;
use core::fmt::Debug;

pub enum BlockFace {
    Top,
    Bottom,
    Front,
    Back,
    Right,
    Left
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