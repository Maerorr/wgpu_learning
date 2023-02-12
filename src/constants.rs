#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub const WIDTH: u32 = 1280;
pub const HEIGHT: u32 = 720;

pub const FOV: f32 = 90.0;

pub const NEAR_CLIP: f32 = 0.1;
pub const FAR_CLIP: f32 = 100.0;

pub const CAM_SPEED: f32 = 0.05;
pub const CAM_ROT_SPEED: f32 = 0.1;