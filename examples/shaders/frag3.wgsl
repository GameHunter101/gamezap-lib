struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
}

@group(2) @binding(0)
var<uniform> num: f32;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    // return textureSample(texture, texture_sampler, in.tex_coords) * coefficient;

    return vec4f(0.0, 0.0, 0.0, 1.0);
}
