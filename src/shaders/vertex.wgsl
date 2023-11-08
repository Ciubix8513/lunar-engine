struct transformations {
  world: mat4x4<f32>,
  view: mat4x4<f32>,
  projection: mat4x4<f32>
}

struct ColorOutput {
  @location(0) tex_coord: vec2<f32>,
  @builtin(position) position: vec4<f32>
}

@group(0) @binding(0) var<uniform> transformation_matrices : transformations;

@vertex
fn main(@location(0) position: vec4<f32>, @location(1) uvs: vec2<f32>, @location(2) normal: vec3<f32>) -> ColorOutput {
    let mat = transpose(transpose(transformation_matrices.world) * transformation_matrices.view * transformation_matrices.projection);
    let o = mat * position;

    // var o = transpose(transformation_matrices.world) * position;
    // o = transformation_matrices.view * o;
    // o = transformation_matrices.projection * o;
    // o.w = 1.0;

    // var o = (transformation_matrices.world * transformation_matrices.projection) * position;

    var res: ColorOutput;
    res.position = o;
    res.tex_coord = uvs;
    return res;
}
