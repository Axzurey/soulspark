use serde::Deserialize;

use crate::blocks::block::Blocks;

#[derive(Deserialize)]
struct StructureData {
    blocks: Vec<Blocks>,
    widthx: u32,
    widthz: u32,
    height: u32,
    schematic: Vec<Vec<u32>>
}