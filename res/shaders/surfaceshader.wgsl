struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) diffuse_texture_index: u32,
    @location(6) normal_texture_index: u32,
    @location(7) emissive_texture_index: u32
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) diffuse_texture_index: u32,
    @location(5) normal_texture_index: u32,
    @location(6) emissive_texture_index: u32,
};

@group(0) @binding(0)
var diffuse_texture_array: binding_array<texture_2d<f32>>;

@group(0) @binding(1)
var diffuse_sampler_array: binding_array<sampler>;

@group(0) @binding(2)
var normal_texture_array: binding_array<texture_2d<f32>>;

@group(0) @binding(3)
var normal_sampler_array: binding_array<sampler>;

@group(0) @binding(4)
var emissive_texture_array: binding_array<texture_2d<f32>>;

@group(0) @binding(5)
var emissive_sampler_array: binding_array<sampler>;

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>
}

@group(1) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.bitangent = model.bitangent;
    out.tangent = model.tangent;
    out.normal = model.normal;
    out.tex_coords = model.tex_coords;
    out.normal_texture_index = model.normal_texture_index;
    out.emissive_texture_index = model.emissive_texture_index;
    out.diffuse_texture_index = model.diffuse_texture_index;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let diffuse_color = textureSampleLevel(diffuse_texture_array[in.diffuse_texture_index], diffuse_sampler_array[in.diffuse_texture_index], in.tex_coords, 0.0).rgba;

    return diffuse_color;
}