use std::collections::{HashMap, VecDeque};

use cgmath::{Vector2, Vector3};

use crate::blocks::block::BlockType;
#[derive(Debug)]
pub enum ChunkAction {
    BreakBlock(Vector3<i32>),
    PlaceBlock(BlockType),
    UpdateChunkLighting(Vector2<i32>),
    UpdateChunkMesh(Vector3<i32>)
}

fn conv(a: &ChunkAction) -> String {
    match a {
        ChunkAction::BreakBlock(v) => format!("Break: {},{},{}", v.x, v.y, v.z),
        ChunkAction::PlaceBlock(v) => {
            let pos = v.get_absolute_position();
            format!("Place: {},{},{},{:?}", pos.x, pos.y, pos.z, v.get_block())
        },
        ChunkAction::UpdateChunkLighting(v) => format!("Lighting: {},{}", v.x, v.y),
        ChunkAction::UpdateChunkMesh(v) => format!("Mesh: {},{},{}", v.x, v.y, v.z),
    }
}

pub struct ChunkActionQueue {
    queue: VecDeque<ChunkAction>,
    map: HashMap<String, bool>
}

impl ChunkActionQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            map: HashMap::new()
        }
    }
    pub fn update_chunk_mesh(&mut self, pos: Vector3<i32>) {
        let action = ChunkAction::UpdateChunkMesh(pos);
        let named: String = conv(&action);

        if !self.map.contains_key(&named) {
            self.queue.push_back(action);
            self.map.insert(named, true);
        }
    }
    pub fn update_chunk_lighting(&mut self, pos: Vector2<i32>) {
        let action = ChunkAction::UpdateChunkLighting(pos);
        let named: String = conv(&action);

        if !self.map.contains_key(&named) {
            self.queue.push_back(action);
            self.map.insert(named, true);
        }
    }
    pub fn place_block(&mut self, block: BlockType) {
        self.queue.push_back(ChunkAction::PlaceBlock(block));
    }
    pub fn break_block(&mut self, position: Vector3<i32>) {
        self.queue.push_back(ChunkAction::BreakBlock(position));
    }
    pub fn get_next_action(&mut self) -> Option<ChunkAction> {
        let action = self.queue.pop_front();

        if action.is_none() {
            return None;
        }

        let act = action.unwrap();

        let named: String = conv(&act);

        if self.map.contains_key(&named) {
            self.map.remove(&named);
        }

        Some(act)
    }
}