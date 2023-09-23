use crate::{mat4x4::Mat4x4, traits::Vector, vec3::Vec3};

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

pub fn scale_matrix(scale: &Vec3) -> Mat4x4 {
    Mat4x4 {
        m00: scale.x,
        m11: scale.y,
        m22: scale.z,
        ..Default::default()
    }
}

pub fn translation_matrix(translation: &Vec3) -> Mat4x4 {
    Mat4x4 {
        m03: translation.x,
        m13: translation.y,
        m23: translation.z,
        ..Default::default()
    }
}

pub fn rotation_matrix_euler(rotation: &Vec3) -> Mat4x4 {
    let sin_x = rotation.x.sin();
    let cos_x = rotation.x.cos();
    let sin_y = rotation.y.sin();
    let cos_y = rotation.y.cos();
    let sin_z = rotation.z.sin();
    let cos_z = rotation.z.cos();

    Mat4x4::new(
        cos_y * cos_z,
        sin_x * sin_y * cos_z - cos_x * cos_z,
        cos_x * sin_y * cos_z + sin_x * sin_z,
        0.0,
        cos_y * sin_z,
        sin_x * sin_y * sin_z + cos_x * cos_z,
        cos_x * sin_y * sin_z - sin_x * cos_z,
        0.0,
        -sin_y,
        sin_x * cos_y,
        cos_x * cos_y,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

pub fn transform_matrix_euler(translation: &Vec3, scale: &Vec3, rotation: &Vec3) -> Mat4x4 {
    scale_matrix(scale) * rotation_matrix_euler(rotation) * translation_matrix(translation)
}

pub fn look_at_matrix(camera_position: Vec3, camera_up: Vec3, camera_forward: Vec3) -> Mat4x4 {
    let zaxis = (camera_forward - camera_position).normalized();
    let xaxis = (zaxis.cross(&camera_up)).normalized();
    let yaxis = zaxis.cross(&xaxis);
    Mat4x4::new(
        xaxis.x,
        yaxis.x,
        zaxis.x,
        0.0,
        xaxis.y,
        yaxis.y,
        zaxis.y,
        0.0,
        xaxis.z,
        yaxis.z,
        zaxis.z,
        0.0,
        -(xaxis.dot_product(&camera_position)),
        -(yaxis.dot_product(&camera_position)),
        -(zaxis.dot_product(&camera_position)),
        1.0,
    )
}
