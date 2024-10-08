@group(0) @binding(0) var texture_input: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(1) var another_input: texture_storage_2d<rgba8unorm, read_write>;

// @group(0) @binding(2) var texture_output: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(2) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // output[global_id.x] = textureLoad(texture_input, vec2i(i32(global_id.x), 0i))[0];
    if (global_id.x + global_id.y) % 2u == 1u {
        textureStore(another_input, vec2i(i32(global_id.x), i32(global_id.y)), vec4f(0.0, 1.0, 0.0, 1.0));
    } else {
        textureStore(another_input, vec2i(i32(global_id.x), i32(global_id.y)), vec4f(1.0, 0.0, 0.0, 1.0));
    }
    /* for (var i: i32 = 0; i < 10; i++) {
        for (var j: i32 = 0; j < 10; j++) {
            textureStore(texture_output, vec2<i32>(i32(global_id.x)*10 + i, i32(global_id.y)*10 + j), vec4f(f32(global_id.x) / 100.0, f32(global_id.y) / 100.0, 0.0, 1.0));
        }
    } */

}
