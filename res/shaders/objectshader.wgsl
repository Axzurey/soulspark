struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tangent_position: vec3<f32>,
    @location(2) tangent_light_position: vec3<f32>,
    @location(3) tangent_view_position: vec3<f32>,
    @location(4) texture_stretch_u: f32,
    @location(5) texture_stretch_v: f32,
    @location(6) t0: vec3<f32>,
    @location(7) t1: vec3<f32>,
    @location(8) t2: vec3<f32>,
    @location(9) diffuse_texture_index: u32
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) diffuse_texture_index: u32
}

struct InstanceInput {
    @location(6) m0: vec4<f32>,
    @location(7) m1: vec4<f32>,
    @location(8) m2: vec4<f32>,
    @location(9) m3: vec4<f32>,
    @location(10) n0: vec3<f32>,
    @location(11) n1: vec3<f32>,
    @location(12) n2: vec3<f32>,
}

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}

struct CurrentLight {
    position: vec4<f32>,
    model: mat4x4<f32>
}

@group(2) @binding(0)
var<uniform> current_light: CurrentLight;

@vertex
fn vs_bake(model: VertexInput, instance: InstanceInput) -> @builtin(position) vec4<f32> {
    let worldmat = mat4x4<f32>(
        instance.m0,
        instance.m1,
        instance.m2,
        instance.m3
    );

    return current_light.model * u_entity.world * vec4<f32>(model.position);
}

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

@group(1) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {

    let size = vec3<f32>(instance.m0.x * 2.0, instance.m1.y * 2.0, instance.m2.z * 2.0);

    let m2matrix = mat4x4<f32>(instance.m0 * 2.0, instance.m1 * 2.0, instance.m2 * 2.0, instance.m3 * 2.0);

    let model_matrix = mat4x4<f32>(instance.m0, instance.m1, instance.m2, instance.m3);

    let normal_matrix = mat3x3<f32>(instance.n0, instance.n1, instance.n2);

    let world_normal = normalize(normal_matrix * model.normal);

    let world_tangent = normalize(normal_matrix * model.tangent);

    let world_bitangent = normalize(normal_matrix * model.bitangent);

    let tangent_matrix = transpose(mat3x3<f32>(
        world_tangent,
        world_bitangent,
        world_normal,
    ));

    let t0 = tangent_matrix[0];
    let t1 = tangent_matrix[1];
    let t2 = tangent_matrix[2];

    let texture_stretch_u = m2matrix * vec4<f32>(world_tangent, 0.0);
    let texture_stretch_v = m2matrix * vec4<f32>(world_bitangent, 0.0);

    let world_position = model_matrix * vec4<f32>(model.position, 1.0);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.tex_coords;
    out.tangent_position = tangent_matrix * world_position.xyz;
    out.tangent_view_position = tangent_matrix * camera.view_pos.xyz;
    out.t0 = t0;
    out.t1 = t1;
    out.t2 = t2;
    out.texture_stretch_u = length(texture_stretch_u);
    out.texture_stretch_v = length(texture_stretch_v);
    out.diffuse_texture_index = model.diffuse_texture_index;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let tex_coords = vec2<f32>(in.tex_coords.x * in.texture_stretch_u, in.tex_coords.y * in.texture_stretch_v);
    
    let object_color: vec4<f32> = textureSample(diffuse_texture_array[in.diffuse_texture_index], diffuse_sampler_array[in.diffuse_texture_index], tex_coords);

    return object_color;
}