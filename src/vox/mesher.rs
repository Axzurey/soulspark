use std::{collections::HashMap, sync::{Arc, RwLock}};

use crate::engine::surfacevertex::SurfaceVertex;

use super::{chunk::Chunk, chunk_manager::get_block_at_absolute_cloned};

struct Mask2D {
    width: usize,
    height: usize,
    data: Vec<bool>,
}

impl Mask2D {
    fn new(width: usize, height: usize) -> Self {
        Mask2D {
            width,
            height,
            data: vec![false; width * height],
        }
    }

    fn get(&self, x: usize, y: usize) -> bool {
        self.data[y * self.width + x]
    }

    fn set(&mut self, x: usize, y: usize, value: bool) {
        self.data[y * self.width + x] = value;
    }

    fn clear(&mut self) {
        for v in &mut self.data {
            *v = false;
        }
    }
}

fn greedy_mesh(
    chunk: Arc<RwLock<Chunk>>,
    slice: u32,
    chunks: HashMap<u32, Arc<RwLock<Chunk>>>,
) -> Vec<[[isize; 3]; 4]> {
    let dims = [16, 16, 16];
    let mut quads = Vec::new();
    let mut vertices: Vec<SurfaceVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    

    quads
}