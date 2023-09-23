@group(0) @binding(0) var<uniform> transformation_matrix : mat4x4<f32>;

@vertex
fn main(@location(0) position: vec4<f32>) -> @builtin(position) vec4<f32> {
    return transformation_matrix * position;
}
