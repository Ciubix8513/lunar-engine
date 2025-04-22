struct Light {
  direction: vec3<f32>,
  intensity: f32,
  color: vec4<f32>,
  ambient_color: vec4<f32>,
}

struct MaterialData {
  color: vec4<f32>,
  specular_color: vec4<f32>,
  shininess: f32,
}

struct PointLight {
  position: vec3<f32>,
  intensity: f32,
  color: vec3<f32>,
  range: f32
}


@group(1)@binding(0)
var<uniform> material: MaterialData;
@group(2)@binding(0)
var<uniform> light: Light;
@group(3)@binding(0)
var<storage, read> point_lights: array<PointLight>;

@fragment
fn main(@builtin(position) pos: vec4<f32>, @location(0) uvs: vec2<f32>, @location(1) normal: vec3<f32>, @location(2) view_dir: vec3<f32>) -> @location(0) vec4<f32> {
    var color = light.ambient_color;
    var specular = vec4(0.0);

    let light_dir = - light.direction;
    let light_intencity = dot(normal, light_dir);

    if light_intencity > 0.0f {

        color += saturate(light.color * light_intencity);

        // reflect(lightlight_dir, normal), but since we already have the dot(x,y) we use this?
        let reflection = normalize(light_dir - 2 * light_intencity * normal);
        specular = material.shininess * (pow(saturate(dot(reflection, view_dir)), 30.0) * material.specular_color);
    }

    color = color * material.color;
    color = saturate(color + specular);

    return color;
}
