use std::sync::Arc;

use super::mesh::Mesh;

pub struct Model {
    pub meshes: Vec<Arc<Mesh>>,
}