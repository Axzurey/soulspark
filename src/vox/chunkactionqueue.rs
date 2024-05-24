use std::collections::VecDeque;

use cgmath::Vector3;

use crate::blocks::block::BlockType;

pub enum ChunkAction {
    BreakBlock(Vector3<i32>),
    PlaceBlock(BlockType)
}

pub struct ChunkActionQueue {
    queue: VecDeque<ChunkAction>
}

impl ChunkActionQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new()
        }
    }

    pub fn place_block(&mut self, block: BlockType) {
        self.queue.push_back(ChunkAction::PlaceBlock(block));
    }
    pub fn break_block(&mut self, position: Vector3<i32>) {
        self.queue.push_back(ChunkAction::BreakBlock(position));
    }
    pub fn get_next_action(&mut self) -> Option<ChunkAction> {
        self.queue.pop_front()
    }
}