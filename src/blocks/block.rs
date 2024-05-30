use cgmath::Vector3;
use serde::Deserialize;
use core::fmt::Debug;
use std::ops::BitOrAssign;

use super::{airblock::AirBlock, dirtblock::DirtBlock, grassblock::GrassBlock, leafblock::LeafBlock, logblock::LogBlock, stoneblock::StoneBlock};

pub type BlockType = Box<dyn Block + Send + Sync>;

pub fn create_block_default(block: Blocks, absolute_position: Vector3<i32>) -> BlockType {
    let relative_position = absolute_position.map(|v| v.rem_euclid(16) as u32);

    match block {
        Blocks::AIR => Box::new(AirBlock::new(relative_position, absolute_position)),
        Blocks::DIRT => Box::new(DirtBlock::new(relative_position, absolute_position)),
        Blocks::GRASS => Box::new(GrassBlock::new(relative_position, absolute_position)),
        Blocks::STONE => Box::new(StoneBlock::new(relative_position, absolute_position)),
        Blocks::LOG => Box::new(LogBlock::new(relative_position, absolute_position)),
        Blocks::LEAF => Box::new(LeafBlock::new(relative_position, absolute_position))
    }
}

#[derive(PartialEq, Eq, Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Blocks {
    AIR,
    DIRT,
    GRASS,
    STONE,
    LOG,
    LEAF
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum BlockFace {
    Top = 0,
    Bottom = 1,
    Right = 2,
    Left = 3,
    Front = 4,
    Back = 5,
}

pub fn calculate_illumination_bytes(block: &BlockType) -> u32 {
    let mut val: u32 = 0;
    
    let sunlight = block.get_sunlight_intensity();
    let light = block.get_light();

    val.bitor_assign(light[0] as u32);
    val.bitor_assign((light[1] as u32) << 8);
    val.bitor_assign((light[2] as u32) << 16);
    val.bitor_assign((sunlight as u32) << 24);

    val
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
    // intensity is from 0-15
    
    fn reset_light(&mut self);
    fn set_sunlight_intensity(&mut self, intensity: u8);
    fn set_light(&mut self, with_color: [u8; 3]);
    fn get_light(&self) -> &[u8; 3];
    fn get_sunlight_intensity(&self) -> u8;
    fn emissive_color(&self) -> Option<[u8; 3]> {None}

    fn get_block(&self) -> Blocks;
}

impl Clone for Box<dyn Block + Send + Sync> {
    fn clone(&self) -> Self {
        create_block_default(self.get_block(), self.get_absolute_position())
    }
    
    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
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