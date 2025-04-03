struct Light {
  direction: vec3<f32>,
  intensity: f32,
  color: vec4<f32>,
  ambient_color: vec4<f32>,
  camera: vec3<f32>,
}

struct MaterialData {
  color: vec4<f32>,
  specular_color: vec4<f32>,
  shininess: f32,
}


@group(1)@binding(0)
var<uniform> material: MaterialData;
@group(2)@binding(0)
var<uniform> light: Light;

@fragment
fn main(@location(0) uvs: vec2<f32>, @location(1) normal: vec3<f32>) -> @location(0) vec4<f32> {
    let n_normal = normalize(normal);

    let base_light = max(dot(light.direction, normal), 0.0);

    let half_dir = normalize(light.direction + light.camera);
    let spec_angle = max(dot(half_dir, normal), 0.0);

    let specular = pow(spec_angle, material.shininess);

    let color = light.ambient_color + base_light * material.color * light.intensity * light.color + specular * material.specular_color * light.intensity * light.color;

    let adjusted = pow(color.xyz, vec3(1.0 / 2.2));

    return vec4(adjusted, 1.0);
}
