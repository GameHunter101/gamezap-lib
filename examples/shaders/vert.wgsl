struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct ModelData {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: Camera;

@vertex
fn main(model: VertexInput, model_data: ModelData) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        model_data.model_matrix_0,
        model_data.model_matrix_1,
        model_data.model_matrix_2,
        model_data.model_matrix_3,
    );

    var out:VertexOutput;

    let world_position = model_matrix * vec4<f32>(model.position,1.0);
    out.clip_position = camera.view_proj * world_position;
    out.color = vec4<f32>(model.normal.xyz, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}
