use cgmath::{Matrix, Matrix3, Matrix4, Quaternion, Rad, Rotation3, SquareMatrix, Vector3, Zero};
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
    pub fn new(transform: Matrix4<f32>) -> Self {
        let model: [[f32; 4]; 4] = transform.into();

        let normal_matrix = mat4_to_mat3(transform);
        let normal_matrix = normal_matrix.transpose();
        let normal_matrix= normal_matrix.invert().unwrap();

        let normal = normal_matrix;

        Self {
            model: model,
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
    pub local: Matrix4<f32>,
    pub world: Matrix4<f32>,
    pub buffer: wgpu::Buffer,
}

impl ModelMatrix {
    pub fn new(device: &wgpu::Device, local_transform: Matrix4<f32>, world_transform: Matrix4<f32>) -> Self {

        let raw_matrix = RawModelMatrix::new(local_transform * world_transform);
        let data = [raw_matrix];
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("model Buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            local: local_transform,
            world: world_transform,
            buffer,
        }
    }

    pub fn identity(device: &wgpu::Device) -> Self {
        let local = Matrix4::identity();
        let world = Matrix4::identity();
        let raw_matrix = RawModelMatrix::new(local*world);
        let data = [raw_matrix];
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("model matrix Buffer"),
                contents: bytemuck::cast_slice(&data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        Self {
            local,
            world,
            buffer,
        }
    }

    pub fn translate_local(
        &mut self,
        position: [f32; 3],
    ) {
        self.local.x.w += position[0];
        self.local.y.w += position[1];
        self.local.z.w += position[2];
    }

    pub fn translate_world(
        &mut self,
        position: [f32; 3],
    ) {
        self.world.x.w += position[0];
        self.world.y.w += position[1];
        self.world.z.w += position[2];
    }

    pub fn scale_local(
        &mut self,
        scale: [f32; 3],
    ) {
        self.local.x.x *= scale[0];
        self.local.y.y *= scale[1];
        self.local.z.z *= scale[2];
    }

    pub fn scale_world(
        &mut self,
        scale: [f32; 3],
    ) {
        self.world.x.x *= scale[0];
        self.world.y.y *= scale[1];
        self.world.z.z *= scale[2];
    }

    pub fn rotate_world(
        &mut self,
        rotation: Quaternion<f32>,
    ) {
        let rotation_matrix = Matrix4::from(rotation);
        self.world = self.world * rotation_matrix;
    }

    pub fn rotate_local(
        &mut self,
        rotation: Quaternion<f32>,
    ) {
        let rotation_matrix = Matrix4::from(rotation);
        self.local = self.local * rotation_matrix;
    }

    pub fn to_raw(&self) -> RawModelMatrix {
        RawModelMatrix::new(self.world * self.local)
    }
}

pub fn mat4_to_mat3(mat: Matrix4<f32>) -> Matrix3<f32> {
    Matrix3::new(
        mat.x.x, mat.x.y, mat.x.z,
        mat.y.x, mat.y.y, mat.y.z,
        mat.z.x, mat.z.y, mat.z.z,
    )
}