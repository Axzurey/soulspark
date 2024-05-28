use std::{collections::HashMap, env, fs::File, io::{BufReader, Read}, sync::RwLock};

use cgmath::Vector3;
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::blocks::block::{create_block_default, BlockType, Blocks};

#[derive(Deserialize)]
struct StructureData {
    pub blocks: Vec<Blocks>,
    pub widthx: u32,
    pub widthz: u32,
    pub height: u32,
    pub schematic: Vec<Vec<u32>>
}
#[derive(Deserialize)]
struct StructurePointerInner {
    weight: f32,
    path: String
}

#[derive(Deserialize)]
struct StructurePointer {
    name: String,
    paths: Vec<StructurePointerInner>,
    density: f32
}

pub fn get_blocks_for_structure_at_point(structure: &str, variant: usize, position: Vector3<i32>) -> Vec<BlockType> {
    let read = LOADED_STRUCTURE_FILES.read().unwrap();
    if !read.contains_key(structure) || read.get(structure).unwrap().len() - 1 < variant {
        println!("{}", format!("Structure: {} doesn't exist with a variant of {}", structure, variant));
    }

    let structure = &read[structure][variant];

    let wx = structure.widthx as i32;
    let wz = structure.widthz as i32;
    let h = structure.height as i32;

    (0..h).flat_map(|y| {
        (-wz / 2..wz / 2 + 1).flat_map(|z| {
            (-wx / 2..wx / 2 + 1).map(|x| {
                let abs = Vector3::new(x, h - 1- y, z) + position;

                let (rx, rz) = (x + wx / 2, z + wz / 2);

                let block_type_index = structure.schematic[y as usize][((rz * wx) + rx) as usize];

                let block_type = structure.blocks[block_type_index as usize];

                let block = create_block_default(block_type, abs);

                block
            }).collect::<Vec<BlockType>>()
        }).collect::<Vec<BlockType>>()
    }).collect::<Vec<BlockType>>()
}

static LOADED_STRUCTURE_FILES: Lazy<RwLock<HashMap<String, Vec<StructureData>>>> = Lazy::new(|| {
    let m = HashMap::new();
    RwLock::new(m)
});

pub fn load_structures() {
    let mut dir = env::current_dir().unwrap();
    dir.push("res/data/structure_manifest.json");

    let file = File::open(dir).expect("Unable to open structure_manifest");
    let reader = BufReader::new(file);
    let data: Vec<StructurePointer> = serde_json::from_reader(reader).expect("Invalid structure_manifest data");

    let mut lock = LOADED_STRUCTURE_FILES.write().unwrap();

    for item in data {
        let mut structs: Vec<StructureData> = Vec::new();

        for path in item.paths {
            let mut dir = env::current_dir().unwrap();
            dir.push("res\\");
            dir.push(path.path);
            let cloned = dir.clone();
            let as_str = cloned.to_str().unwrap();

            let file = File::open(dir).expect(&format!("Unable to open {}", as_str));
            let reader = BufReader::new(file);
            let data: StructureData = serde_json::from_reader(reader).expect(&format!("{} does not have correct formatting", as_str));
            structs.push(data);
        }
        lock.insert(item.name, structs);
    }
}