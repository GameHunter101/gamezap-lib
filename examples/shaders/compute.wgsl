@group(0) @binding(0) var<storage, read_write> data: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;

fn work(num: u32) -> u32 {
    var output:u32 = num * 2u;
    return output;
}

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    output[global_id.x] = work(data[global_id.x]);
}
