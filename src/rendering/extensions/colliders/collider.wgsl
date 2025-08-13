
@fragment
fn main(@builtin(position) pos: vec4<f32>, @location(0) uvs: vec2<f32>, @location(1) normal: vec3<f32>, @location(2) view_dir: vec3<f32>, @location(3) world_pos: vec3<f32>) -> @location(0) vec4<f32> {
  return vec4(1.0);
}
