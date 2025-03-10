struct Light {
  direction: vec3<f32>,
  intensity: f32,
  color: vec4<f32>,
  ambient_color: vec4<f32>,
  camera: vec3<f32>,
}

struct MaterialData {
  color: vec4<f32>,
  shininess: f32,
}


@group(1)@binding(0)
var<uniform> material: MaterialData;
@group(2)@binding(0)
var<uniform> light: Light;

@fragment
fn main(@location(0) uvs: vec2<f32>, @location(1) normal: vec3<f32>) -> @location(0) vec4<f32> {
    let normal_len = length(normal);

    //We can assume that both of these are already normalized, therefore it is just cos(i)
    let angle = clamp(dot(normal, -light.direction) / normal_len, 0.0, 1.0);

    //Abs so it doesn't go into the negative
    let modified_light_color = clamp(light.color - light.ambient_color, vec4(0.0), vec4(1.0));

    let light_color = (angle * modified_light_color) * material.color;
    let out_color = (light.ambient_color * material.color) + light_color;


    let light_reflection = reflect(-light.direction, normal);

    let  specular = clamp(material.shininess * pow(dot(light_reflection, light.camera) / length(light_reflection), 20.0), 0.0, 100000.0);

    return vec4(out_color.xyz + vec3(specular), 1.0);
    // return vec4(specular);
}
