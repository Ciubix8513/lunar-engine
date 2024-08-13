@group(1)@binding(0)
var color: vec4<f32>;

@fragment
fn main() -> @location(0) vec4<f32> {
    return color;
}
