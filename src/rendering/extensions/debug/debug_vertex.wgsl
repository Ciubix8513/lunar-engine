struct vs_out { 
  @location(0) color: vec4f,
  @builtin(position) pos: vec4f,
}

struct Camera {
  matrix: mat4x4<f32>,
  t_matrix: mat4x4<f32>,
  position: vec3<f32>
}

@group(0) @binding(0) var<uniform> camera: Camera;

@vertex
fn main(@location(0) pos: vec3f, @location(1) color: vec4f) -> vs_out {
    var res: vs_out;

    res.pos = camera.matrix * vec4(pos, 1);
    res.color = color;

    return res;
}
