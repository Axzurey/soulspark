use cgmath::Vector3;

pub trait Block {
    /**
     the block's position in the world
    */
    fn get_absolute_position(&self) -> Vector3<i32>;
    /**
     the block's position in the (sub)chunk
    */
    fn get_relative_position(&self) -> Vector3<u32>;
}