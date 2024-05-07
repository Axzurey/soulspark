use cached::proc_macro::cached;
use cgmath::Vector2;

#[cached]
pub fn local_xyz_to_index(x: u32, y: u32, z: u32) -> usize {
    ((z * 16 * 16) + (y * 16) + x) as usize
}

#[cached]
pub fn xz_to_index(x: i32, z: i32) -> usize {
    let x0 = if x >= 0 {2 * x} else {-2 * x - 1}; //converting integers to natural numbers
    let z0 = if z >= 0 {2 * z} else {-2 * z - 1};

    (0.5 * (x0 + z0) as f32 * (x0 + z0 + 1) as f32 + z0 as f32) as usize //cantor pairing https://math.stackexchange.com/questions/3003672/convert-infinite-2d-plane-integer-coords-to-1d-number
}

pub struct Chunk {
    pub position: Vector2<i32>,
    
}