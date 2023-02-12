// Vertex shader

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
};

struct ModelMatrix {
    @location(3) model_matrix_0: vec4<f32>,
    @location(4) model_matrix_1: vec4<f32>,
    @location(5) model_matrix_2: vec4<f32>,
    @location(6) model_matrix_3: vec4<f32>,

    @location(7) normal_matrix_0: vec3<f32>,
    @location(8) normal_matrix_1: vec3<f32>,
    @location(9) normal_matrix_2: vec3<f32>,
}

struct CameraUniform {
    view_projection: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(2) @binding(0)
var<uniform> light: Light;

@vertex
fn vs_main(
    vertex_input: VertexInput,
    model: ModelMatrix,
) -> VertexOutput {

    // MODEL MATRIX
    let model_matrix = mat4x4<f32>(
        model.model_matrix_0,
        model.model_matrix_1,
        model.model_matrix_2,
        model.model_matrix_3,
    );
    // NORMAL MATRIX
    let normal_matrix = mat3x3<f32>(
        model.normal_matrix_0,
        model.normal_matrix_1,
        model.normal_matrix_2,
    );

    var out: VertexOutput;
    out.tex_coords = vertex_input.tex_coords;
    out.world_normal = normal_matrix * vertex_input.normal;
    var world_position: vec4<f32> = model_matrix * vec4<f32>(vertex_input.pos, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_projection * world_position;
    return out;
}

// Fragment shader

// bind group nr. 0
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let a_s = 0.05;
    let a = a_s * light.color;

    let light_dir = normalize(light.position - in.world_position);

    let d_s = max(dot(in.world_normal, light_dir), 0.0);
    let d = d_s * light.color;

    let texture_col = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let result = (a + d) * texture_col.xyz;

    return vec4<f32>(result, texture_col.a);
}