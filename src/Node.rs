use cgmath::{Matrix4, Quaternion, SquareMatrix, Vector3};
use crate::model_matrix::ModelMatrix;

pub struct Node {
    pub name: String,
    model_matrix: ModelMatrix
}

impl Node {
    pub fn new(device: &wgpu::Device, name: String, world_position: Matrix4<f32>) -> Self {
        Self {
            name,
            model_matrix: ModelMatrix::new(device, Matrix4::identity(), world_position)
        }
    }
}