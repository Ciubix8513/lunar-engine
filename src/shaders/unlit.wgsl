struct MaterialData {
  color: vec4<f32>,
}

struct Camera {
  matrix: mat4x4<f32>,
  t_matrix: mat4x4<f32>,
  position: vec3<f32>
}

@group(1)@binding(0)
var<uniform> material: MaterialData;

@group(1)@binding(1)
var texture: texture_2d<f32>;
@group(1)@binding(2)
var tex_sampler: sampler;


@fragment
fn main(@builtin(position) pos: vec4<f32>, @location(0) uvs: vec2<f32>, @location(1) normal: vec3<f32>, @location(2) view_dir: vec3<f32>, @location(3) world_pos: vec3<f32>) -> @location(0) vec4<f32> {
    let color = material.color * textureSample(texture, tex_sampler, uvs);

    return color;
}
