use std::{fmt::Display, sync::Arc};

use cached::{cached_key, proc_macro::cached, SizedCache};
use cgmath::{Quaternion, Vector3};
use wgpu::util::DeviceExt;

use crate::engine::{mesh::Mesh, model::Model, texture_loader::get_indices_from_texture, vertex::{calculate_tangents_inplace_modelvertex, ModelVertex}};

use super::object::Object;

#[derive(PartialEq)]
pub enum Primitive {
    Ball {
        radius: f32,
        subdivisions: u32
    },
    Rect {
        min: Vector3<f32>,
        max: Vector3<f32>
    },
    Cube
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Ball { radius, subdivisions } => 
                write!(f, "Primitive(Ball)(Radius: {}, Subdivisions{})", radius, subdivisions),
            Primitive::Rect { min, max } => 
                write!(f, "Primitive(Rect)(Min: {}, {}, {}, Max: {}, {}, {})", min.x, min.y, min.z, max.x, max.y, max.z),
            Primitive::Cube => write!(f, "Primitive(Cube)"),
        }
    }
}


#[cached]
fn create_cube(diffuse_texture_index: u32) -> (Vec<ModelVertex>, Vec<u32>) {
    let mut vertices: Vec<ModelVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::from([
        2, 1, 0, 3, 2, 0,
        4, 5, 6, 4, 6, 7,
        8, 9, 10, 8, 10, 11,
        14, 13, 12, 15, 14, 12,
        16, 17, 18, 16, 18, 19,
        22, 21, 20, 23, 22, 20
    ]);

    let half_side_length = 0.5;
    
    //back
    vertices.push(ModelVertex {position: [-half_side_length, -half_side_length, -half_side_length], tex_coords: [0., 0.],
        normal: [0., 0., -1.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [half_side_length, -half_side_length, -half_side_length], tex_coords: [1., 0.],
        normal: [0., 0., -1.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [half_side_length, half_side_length, -half_side_length], tex_coords: [1., 1.],
        normal: [0., 0., -1.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [-half_side_length, half_side_length, -half_side_length], tex_coords: [0., 1.],
        normal: [0., 0., -1.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    //front
    vertices.push(ModelVertex {position: [-half_side_length, -half_side_length, half_side_length], tex_coords: [0., 0.],
        normal: [0., 0., 1.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [half_side_length, -half_side_length, half_side_length], tex_coords: [1., 0.],
        normal: [0., 0., 1.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [half_side_length, half_side_length, half_side_length], tex_coords: [1., 1.],
        normal: [0., 0., 1.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [-half_side_length, half_side_length, half_side_length], tex_coords: [0., 1.],
        normal: [0., 0., 1.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    //left
    vertices.push(ModelVertex {position: [-half_side_length, -half_side_length, -half_side_length], tex_coords: [0., 0.],
        normal: [-1., 0., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [-half_side_length, -half_side_length, half_side_length], tex_coords: [1., 0.],
        normal: [-1., 0., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [-half_side_length, half_side_length, half_side_length], tex_coords: [1., 1.],
        normal: [-1., 0., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [-half_side_length, half_side_length, -half_side_length], tex_coords: [0., 1.],
        normal: [-1., 0., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    //right
    vertices.push(ModelVertex {position: [half_side_length, -half_side_length, -half_side_length], tex_coords: [0., 0.],
        normal: [1., 0., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [half_side_length, -half_side_length, half_side_length], tex_coords: [1., 0.],
        normal: [1., 0., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [half_side_length, half_side_length, half_side_length], tex_coords: [1., 1.],
        normal: [1., 0., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [half_side_length, half_side_length, -half_side_length], tex_coords: [0., 1.],
        normal: [1., 0., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    //bottom
    vertices.push(ModelVertex {position: [-half_side_length, -half_side_length, -half_side_length], tex_coords: [0., 0.],
        normal: [0., -1., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [half_side_length, -half_side_length, -half_side_length], tex_coords: [1., 0.],
        normal: [0., -1., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [half_side_length, -half_side_length, half_side_length], tex_coords: [1., 1.],
        normal: [0., -1., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [-half_side_length, -half_side_length, half_side_length], tex_coords: [0., 1.],
        normal: [0., -1., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    //top
    vertices.push(ModelVertex {position: [-half_side_length, half_side_length, -half_side_length], tex_coords: [0., 0.],
        normal: [0., 1., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [half_side_length, half_side_length, -half_side_length], tex_coords: [1., 0.],
        normal: [0., 1., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [half_side_length, half_side_length, half_side_length], tex_coords: [0., 1.],
        normal: [0., 1., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});
    vertices.push(ModelVertex {position: [-half_side_length, half_side_length, half_side_length], tex_coords: [0., 0.],
        normal: [0., 1., 0.], bitangent: [0., 0., 0.], tangent: [0., 0., 0.], diffuse_texture_index});

    calculate_tangents_inplace_modelvertex(&mut vertices, &mut indices);

    (vertices, indices)
    
}

cached_key! {
    LENGTH: SizedCache<String, Arc<Mesh>> = SizedCache::with_size(100);
    Key = { format!("{}:{}", p, diffuse_texture_index) };
    fn create_primitive(p: Primitive, device: &wgpu::Device, diffuse_texture_index: u32) -> Arc<Mesh> = {
        println!("{}", diffuse_texture_index);
        match p {
            Primitive::Cube => {
                let (mut vertices, mut indices) = create_cube(diffuse_texture_index);
                calculate_tangents_inplace_modelvertex(&mut vertices, &mut indices);

                for i in 0..3 {
                    println!("{:?}", vertices[i]);
                }
    
                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Chunk Vertex Buffer")),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
    
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Index Buffer")),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
    
                Arc::new(Mesh {
                    vertex_buffer: vertex_buffer,
                    index_buffer: index_buffer,
                    num_elements: indices.len() as u32,
                    vertices: vertices,
                    face_indices: indices
                })
            },
            _ => {todo!()}
        }
    }
}

pub struct PrimitiveBuilder {
    mesh: Option<Arc<Mesh>>,
    position: Vector3<f32>,
    size: Vector3<f32>,
    rotation: Quaternion<f32>,
    diffuse_texture_index: Option<u32>
}

impl PrimitiveBuilder {
    pub fn new() -> Self {
        Self {
            mesh: None,
            rotation: Quaternion::new(1., 0., 0., 0.),
            size: Vector3::new(1., 1., 1.),
            position: Vector3::new(0., 0., 0.),
            diffuse_texture_index: None
        }
    }

    pub fn set_position(mut self, position: Vector3<f32>) -> Self {
        self.position = position;

        self
    }
    pub fn set_size(mut self, size: Vector3<f32>) -> Self {
        self.size = size;

        self
    }
    pub fn set_orientation(mut self, orientation: Quaternion<f32>) -> Self {
        self.rotation = orientation;

        self
    }

    pub fn set_diffuse_texture_by_name(mut self, name: &str) -> Self {
        let diffuse_texture_index = get_indices_from_texture(name);
        
        self.diffuse_texture_index = Some(diffuse_texture_index as u32);

        self
    }

    pub fn finalize(&self) -> Object {
        let mut object = Object::new("some object".to_owned(), Arc::new(Model {
            meshes: [self.mesh.clone().expect("set_primitive must be called before finalize")].to_vec()
        }));

        object.set_orientation(self.rotation);
        object.set_position(self.position);
        object.set_scale(self.size);

        object
    }

    pub fn set_primitive(mut self, device: &wgpu::Device, p: Primitive) -> Self {
        
        let mesh = create_primitive(p, device, self.diffuse_texture_index.expect("all textures must be set before set_primitive is called"));

        self.mesh = Some(mesh);

        self
    }
}