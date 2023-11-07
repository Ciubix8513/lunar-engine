struct transformations {
  object: mat4x4<f32>,
  camera: mat4x4<f32>,
  screen: mat4x4<f32>
}

struct ColorOutput {
  @location(0) tex_coord: vec2<f32>,
  @builtin(position) position: vec4<f32>
}

@group(0) @binding(0) var<uniform> transformation_matrices : transformations;

@vertex
fn main(@location(0) position: vec3<f32>, @location(1) uvs: vec2<f32>, @location(2) normal: vec3<f32>) -> ColorOutput {
    var o = transformation_matrices.object * vec4<f32>(position, 1.0);
    o = transformation_matrices.camera * o;
    o = transformation_matrices.screen * o;
    o.w = 1.0;
    var res: ColorOutput;
    res.position = o;
    res.tex_coord = uvs;
    return res;
}
