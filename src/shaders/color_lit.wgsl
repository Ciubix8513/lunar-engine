struct Light {
  direction: vec3<f32>,
  intensity: f32,
  color: vec4<f32>,
}


@group(1)@binding(0)
var<uniform> color: vec4<f32>;
@group(2)@binding(0)
var<uniform> light: Light;

@fragment
fn main(@location(0) uvs: vec2<f32>, @location(1) normal: vec3<f32>) -> @location(0) vec4<f32> {
    return light.color;
}
