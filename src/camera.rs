use cgmath::{InnerSpace, SquareMatrix, Vector3};
use wgpu::Device;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::window::Window;
use crate::constants::{CAM_ROT_SPEED, CAM_SPEED, FAR_CLIP, FOV, HEIGHT, NEAR_CLIP, OPENGL_TO_WGPU_MATRIX, WIDTH};

pub struct Camera {
    position: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    pub aspect: f32,

    pub uniform: CameraUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(device: &wgpu::Device) -> Self {
        let position = cgmath::Point3::new(0.0, 0.0, 2.0);
        let target = cgmath::Point3::new(0.0, 0.0, -1.0);
        let up = cgmath::Vector3::new(0.0, 1.0, 0.0);
        let aspect = WIDTH as f32 / HEIGHT as f32;

        let view = cgmath::Matrix4::look_at_rh(
            position,
            target,
            up);
        let projection = cgmath::perspective(
            cgmath::Deg(FOV),
            aspect,
            NEAR_CLIP,
            FAR_CLIP);

        let matrix = OPENGL_TO_WGPU_MATRIX * projection * view;

        let camera_uniform = CameraUniform::new(
            matrix
        );

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform.view_proj]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &create_camera_bind_group_layout(device),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }
                ],
        });

        Self {
            position,
            target,
            up,
            aspect: WIDTH as f32 / HEIGHT as f32,
            uniform: camera_uniform,
            buffer,
            bind_group,
        }
    }

    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(
            self.position,
            self.target,
            self.up);
        let projection = cgmath::perspective(
            cgmath::Deg(FOV),
            self.aspect,
            NEAR_CLIP,
            FAR_CLIP);

        OPENGL_TO_WGPU_MATRIX * projection * view
    }

    pub fn update_view_proj(&mut self, device: &Device) {
        self.uniform.view_proj = self.build_view_projection_matrix().into();
        self.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[self.uniform.view_proj]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        self.bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &create_camera_bind_group_layout(device),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.buffer.as_entire_binding(),
                    }
                ],
            });
    }
}

pub fn create_camera_bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ],
    })
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4]
}

impl CameraUniform {
    pub fn new(view_proj_matrix: cgmath::Matrix4<f32>) -> Self {
        Self {
            view_proj: view_proj_matrix.into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    speed: f32,
    rot_speed: f32,
    left_press: bool,
    right_press: bool,
    up_press: bool,
    down_press: bool,
    rotate_left: bool,
    rotate_right: bool,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            speed: CAM_SPEED,
            rot_speed: CAM_ROT_SPEED,
            left_press: false,
            right_press: false,
            up_press: false,
            down_press: false,
            rotate_left: false,
            rotate_right: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.up_press = is_pressed;
                        //println!("up: {}", self.up_press);
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.left_press = is_pressed;
                        //println!("left: {}", self.left_press);
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.down_press = is_pressed;
                        //println!("down: {}", self.down_press);
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.right_press = is_pressed;
                        //println!("right: {}", self.right_press);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = (camera.target - camera.position).normalize();
        let mut move_val: Vector3<f32>;
        if self.up_press {
            move_val = forward * self.speed;
            camera.position += move_val;
            camera.target += move_val;
        }
        if self.down_press {
            move_val = forward * self.speed;
            camera.position -= move_val;
            camera.target -= move_val;
        }
        if self.left_press {
            move_val = forward.cross(camera.up).normalize() * self.speed;
            camera.position -= move_val;
            camera.target -= move_val;
        }
        if self.right_press {
            move_val = forward.cross(camera.up).normalize() * self.speed;
            camera.position += move_val;
            camera.target += move_val;
        }

    }
}