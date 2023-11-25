// struct transformations {
//   world: mat4x4<f32>,
//   view: mat4x4<f32>,
//   projection: mat4x4<f32>
// }

struct ColorOutput {
  @location(0) tex_coord: vec2<f32>,
  @builtin(position) position: vec4<f32>
}

@group(0) @binding(0) var<uniform> trans_mat: mat4x4;
@group(1) @binding(0) var<uniform> camera: mat4x4;

@vertex
fn main(@location(0) position: vec4<f32>, @location(1) uvs: vec2<f32>, @location(2) normal: vec3<f32>) -> ColorOutput {
    // let mat = trans_mat.projection * trans_mat.view * trans_mat.world ;
    var o = trans_mat * position;
    o = camera * o;

    var res: ColorOutput;
    res.position = o;
    res.tex_coord = uvs;
    return res;
}
