@group(0) @binding(0) var texture_input: texture_2d<f32>;

// @group(0) @binding(1) var texture_output: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    output[global_id.x] = global_id.x;
    /* for (var i: i32 = 0; i < 10; i++) {
        for (var j: i32 = 0; j < 10; j++) {
            textureStore(texture_output, vec2<i32>(i32(global_id.x)*10 + i, i32(global_id.y)*10 + j), vec4f(f32(global_id.x) / 100.0, f32(global_id.y) / 100.0, 0.0, 1.0));
        }
    } */

}
