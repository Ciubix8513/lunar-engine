struct Light {
  direction: vec3<f32>,
  intensity: f32,
  color: vec4<f32>,
  ambient_color: vec4<f32>
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
    //We can assume that both of these are already normalized, therefore it is just cos(i)
    let angle = clamp(dot(normal, -light.direction), 0.0, 1.0);

    //Abs so it doesn't go into the negative
    let modified_light_color = clamp(light.color - light.ambient_color, vec4(0.0), vec4(1.0));

    let light_color = (angle * modified_light_color) * color;
    let out_color = (light.ambient_color * color) + light_color;


    return vec4(out_color.xyz, 1.0);
}
