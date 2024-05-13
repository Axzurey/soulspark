use std::{fs, path::Path};

pub fn create_render_pipeline(
    name: &str,
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader_path: &str,
    backface_culling: bool,
    depth_write_enabled: bool,
    bias: Option<wgpu::DepthBiasState>,
    nocolor: bool,
    nofrag: bool,
    bake: bool
) -> wgpu::RenderPipeline {
    let shader_descriptor = wgpu::ShaderModuleDescriptor {
        label: Some(shader_path),
        source: wgpu::ShaderSource::Wgsl(fs::read_to_string(Path::new(shader_path)).unwrap().into())
    };

    let shader = device.create_shader_module(shader_descriptor);

    let color_targetstate = [Some(wgpu::ColorTargetState {
        format: color_format,
        blend: Some(wgpu::BlendState {
            alpha: wgpu::BlendComponent::OVER,
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add
            },
        }),
        write_mask: wgpu::ColorWrites::ALL,
    })];

    let desc = wgpu::RenderPipelineDescriptor {
        label: Some(name),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: if bake {"vs_bake"} else {"vs_main"},
            buffers: vertex_layouts,
        },
        fragment: if nofrag {None} else {Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: if nocolor {&[]} else {&color_targetstate},
        })},
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: if backface_culling {Some(wgpu::Face::Back)} else {None},
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: true,
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: if bias.is_some() {bias.unwrap()} else {wgpu::DepthBiasState::default()},
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    };

    device.create_render_pipeline(&desc)
}