### 0 
@group(3)@binding(1)
var<storage, read> point_lights: array<PointLight>;
### 1
@group(3)@binding(1)
var<uniform> point_lights: array<PointLight, 256>;
###


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

struct Camera {
  matrix: mat4x4<f32>,
  t_matrix: mat4x4<f32>,
  position: vec3<f32>
}

//gonna keep it bc why not
@group(0) @binding(0) var<uniform> camera: Camera;

@group(1)@binding(0)
var<uniform> material: MaterialData;

@group(1)@binding(1)
var texture: texture_2d<f32>;
@group(1)@binding(2)
var tex_sampler: sampler;
@group(2)@binding(0)
var<uniform> directional_light: Light;

@group(3)@binding(0)
var<uniform> num_lights: u32;

@fragment
fn main(@builtin(position) pos: vec4<f32>, @location(0) uvs: vec2<f32>, @location(1) normal: vec3<f32>, @location(2) view_dir: vec3<f32>, @location(3) world_pos: vec3<f32>) -> @location(0) vec4<f32> {
    var color = directional_light.ambient_color;
    var specular = vec4(0.0);

    let len = num_lights;

    for (var i: u32 = 0; i < len; i++) {
        let dir = point_lights[i].position - world_pos;
        let distance = length(dir);

        if distance > point_lights[i].range {
          continue;
        }

        //Normalize using already calculated distance
        let l_dir = dir / distance;

        //Inverse square law
        let intensity = dot(normal, l_dir);

        if intensity == 0.0 {
          continue;
        }

        let attenuation = saturate(1.0 / (distance * distance)); //* point_lights[i].intensity;
        color += saturate(vec4(point_lights[i].color, 1.0) * intensity * point_lights[i].intensity * attenuation);

        let refl = normalize((l_dir - 2 * intensity * normal));
        specular += saturate(material.shininess * attenuation * (pow(dot(refl, view_dir), 30.0) * material.specular_color));
    }

    let light_dir = - directional_light.direction;
    let light_intencity = dot(normal, light_dir);

    if light_intencity > 0.0 {

        color += saturate(directional_light.color * light_intencity * directional_light.intensity);

        // reflect(lightlight_dir, normal), but since we already have the dot(x,y) we use this?
        let reflection = normalize(light_dir - 2 * light_intencity * normal);
        specular += material.shininess * directional_light.intensity * (pow(saturate(dot(reflection, view_dir)), 30.0) * material.specular_color);
    }

    color = color * material.color * textureSample(texture, tex_sampler, uvs);
    color = saturate(color + specular);

    return color;
}
