use cgmath::{Matrix3, Matrix4, Quaternion, Rad, Rotation3, SquareMatrix, Vector3};
use wgpu::util::DeviceExt;
use wgpu::VertexBufferLayout;
use crate::model::{ModelVertex, Vertex};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawModelMatrix {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

impl RawModelMatrix {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>) -> Self {
        let model =
            Matrix4::from_translation(position) *
            Matrix4::from(rotation);
        let normal = Matrix3::from(rotation);

        Self {
            model: model.into(),
            normal: normal.into(),
        }
    }

    pub fn identity() -> Self {
        Self {
            model: cgmath::Matrix4::identity().into(),
            normal: cgmath::Matrix3::identity().into(),
        }
    }
}

impl Vertex for RawModelMatrix {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        const ATTRIBS: [wgpu::VertexAttribute; 7] =
            wgpu::vertex_attr_array![
                3 => Float32x4,
                4 => Float32x4,
                5 => Float32x4,
                6 => Float32x4,
                7 => Float32x3,
                8 => Float32x3,
                9 => Float32x3,
            ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RawModelMatrix>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBS,
        }
    }
}

pub struct ModelMatrix {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub buffer: wgpu::Buffer,
}

impl ModelMatrix {
    pub fn new(device: &wgpu::Device, position: Vector3<f32>, rotation: Quaternion<f32>) -> Self {
        let raw_matrix = RawModelMatrix::new(position, rotation);
        let data = [raw_matrix];
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("model Buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            position,
            rotation,
            buffer,
        }
    }

    pub fn to_raw(&self) -> RawModelMatrix {
        RawModelMatrix::new(self.position, self.rotation)
    }
}