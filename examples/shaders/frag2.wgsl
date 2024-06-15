struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

@group(2) @binding(0)
var<uniform> coefficient: f32;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, in.tex_coords) * coefficient;
}
