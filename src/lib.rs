mod graphics_context;
mod texture;
mod simple_pipeline;
mod vertex;
mod camera;
mod constants;
mod model;
mod light;
mod model_matrix;
mod Node;

use cgmath::{Quaternion, Rotation3, Vector3};
use wgpu::{FragmentState, include_wgsl, VertexState};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::window::Window;
use crate::camera::{Camera, CameraController};
use crate::constants::{HEIGHT, WIDTH};
use crate::graphics_context::GraphicsContext;
use crate::light::{create_light_pipeline, DrawLight, Light};
use crate::model::{DrawModel, load_model, Model};
use crate::model_matrix::ModelMatrix;
use crate::simple_pipeline::SimplePipeline;
use crate::vertex::Vertex;
use crate::texture::{load_texture, Texture};

struct State {
    ctx: GraphicsContext,
    pipeline: SimplePipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    texture: Texture,

    camera: Camera,
    camera_controller: CameraController,

    obj_model: Model,

    light_model: Model,

    light: Light,
    light_pipeline: wgpu::RenderPipeline,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window) -> Self {
        let context = GraphicsContext::new(window).await;

        let pipeline = SimplePipeline::new(
            &context.device,
            &context.config,
        );

        let vertex_buffer = Vertex::create_vertex_buffer(&context.device);
        let index_buffer = Vertex::create_index_buffer(&context.device);

        let texture = load_texture(
            "textures",
            "default.jpg",
            &context.device,
            &context.queue,
            false
        ).unwrap();

        let camera = Camera::new(&context.device);
        let camera_controller = CameraController::new();

        let obj_model = load_model("models\\blob\\", "blob.obj", &context.device, &context.queue, texture.get_layout())
                .await
                .unwrap();

        let light_model = load_model("models\\d20\\", "d20.obj", &context.device, &context.queue, texture.get_layout())
                .await
                .unwrap();

        let light = Light::new(
            &context.device,
            Vector3::new(5.0, 0.0, 0.0),
            Vector3::new(0.4, 0.8, 0.6));

        let light_pipeline = create_light_pipeline(
            &context.device,
            &context.config,
        );

        Self {
            ctx: context,
            pipeline,
            vertex_buffer,
            index_buffer,
            texture,
            camera,
            camera_controller,
            obj_model,
            light_model,
            light,
            light_pipeline,
        }
    }

    pub fn window(&self) -> &Window {
        &self.ctx.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.ctx.size = new_size;
            self.ctx.config.width = new_size.width;
            self.ctx.config.height = new_size.height;
            self.ctx.surface.configure(&self.ctx.device, &self.ctx.config);

            let new_depth = texture::Texture::create_depth_texture(&self.ctx.device, &self.ctx.config, "depth texture");
            self.ctx.depth_texture = new_depth;

            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
            self.camera.update_view_proj(&self.ctx.device);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera.update_view_proj(&self.ctx.device);
        self.ctx.queue.write_buffer(
            &self.camera.buffer,
            0,
            bytemuck::cast_slice(&[self.camera.uniform]),
        );
        let pos: Vector3<_> = self.light.uniform.position.into();
        self.light.uniform.position = (cgmath::Quaternion::from_axis_angle(
            cgmath::Vector3::unit_y(),
            cgmath::Deg(1.0)
        ) * pos).into();

        self.ctx.queue.write_buffer(
            &self.light.buffer,
            0,
            bytemuck::cast_slice(&[self.light.uniform]),
        );

        let rotation = Quaternion::from_axis_angle(Vector3::unit_y(), cgmath::Deg(0.2));
        self.obj_model.rotate_world(rotation);
        self.ctx.queue.write_buffer(
            &self.obj_model.model_matrix.buffer,
            0,
            bytemuck::cast_slice(&[self.obj_model.model_matrix.to_raw()]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let out = self.ctx.surface.get_current_texture()?;
        let view = out.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.ctx.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            }
        );

        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("render pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(
                                    wgpu::Color {
                                        r: 0.1,
                                        g: 0.5,
                                        b: 0.9,
                                        a: 1.0,
                                    }
                                ),
                                store: true,
                            },
                        }
                    )],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachment {
                            view: &self.ctx.depth_texture.view,
                            depth_ops: Some(
                                wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(1.0),
                                    store: true,
                                }
                            ),
                            stencil_ops: None,
                        }
                    )
                }
            );

            // all rendering things come here:


            render_pass.set_pipeline(&self.light_pipeline);
            render_pass.draw_light_model(
                &self.light_model,
                &self.camera.bind_group,
                &self.light.bind_group,
            );

            render_pass.set_pipeline(&self.pipeline.render_pipeline);
            render_pass.set_vertex_buffer(1, self.obj_model.model_matrix.buffer.slice(..));
            render_pass.draw_model(
                &self.obj_model,
                &self.camera.bind_group,
            &self.light.bind_group);

        }
        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        out.present();
        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_inner_size(winit::dpi::LogicalSize::new(WIDTH, HEIGHT));

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {

            // main rendering event
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.ctx.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            },


            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}
