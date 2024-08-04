@group(0) @binding(0) var<storage, read_write> data: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

fn work(num: f32) -> f32 {
    var output:f32 = num*1.1;
    return output;
}

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    output[global_id.x] = work(data[global_id.x]);
    data[global_id.x] = work(data[global_id.x]);
}
