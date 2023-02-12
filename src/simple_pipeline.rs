use wgpu::{FragmentState, VertexState};
use crate::camera::create_camera_bind_group_layout;
use crate::graphics_context::GraphicsContext;
use crate::light::create_light_bind_group_layout;
use crate::model::{ModelVertex, Vertex};
use crate::model_matrix::{ModelMatrix, RawModelMatrix};
use crate::texture::{create_texture_bind_group_layout};
pub struct SimplePipeline {
    pub render_pipeline: wgpu::RenderPipeline,
}

impl SimplePipeline {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {

        // all the bind groups layouts used by this pipeline
        let layouts = &[
            &create_texture_bind_group_layout(device),
            &create_camera_bind_group_layout(device),
            &create_light_bind_group_layout(device),
        ];

        let layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: layouts,
                push_constant_ranges: &[],
            }
        );

        let shader = device.create_shader_module(
            wgpu::include_wgsl!("../res/shaders/shader.wgsl")
        );

        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[ModelVertex::desc(), RawModelMatrix::desc()],
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // 2.
                    cull_mode: None,//Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: Default::default(),
                multiview: None,

            }
        );

        Self {
            render_pipeline,
        }
    }
}