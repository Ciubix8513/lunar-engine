use crate::mat4x4::Mat4x4;

pub fn perspercive_projection(
    fov: f32,
    screen_aspect: f32,
    screen_near: f32,
    screen_far: f32,
) -> Mat4x4 {
    let y_scale = 1.0 / (fov.tan() / 2.0);
    let x_scale = y_scale / screen_aspect;

    let c = screen_far / (screen_far - screen_near);

    let d = -screen_near * c;

    Mat4x4::new(
        x_scale, 0.0, 0.0, 0.0, 0.0, y_scale, 0.0, 0.0, 0.0, 0.0, c, 1.0, 0.0, 0.0, d, 0.0,
    )
}
