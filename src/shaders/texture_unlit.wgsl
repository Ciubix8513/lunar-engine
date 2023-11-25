@group(1)@binding(0)
var texture: texture_2d<f32>;
@group(1)@binding(1)
var tex_sampler: sampler;

@fragment
fn main(@location(0) uvs: vec2<f32>) -> @location(0) vec4<f32> {
    let col = textureSample(texture, tex_sampler, uvs);
    return col;
}
