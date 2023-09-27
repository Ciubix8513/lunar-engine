
struct transformations {
  object: mat4x4<f32>,
  camera: mat4x4<f32>,
  screen: mat4x4<f32>
}

@group(0) @binding(0) var<uniform> transformation_matrices : transformations;

@vertex
fn main(@location(0) position: vec4<f32>) -> @builtin(position) vec4<f32> {
    var o = transformation_matrices.object * position;
    o.w = 0.0;
    return o;
}
