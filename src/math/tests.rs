use super::*;

#[test]
fn test_matrix_float_mul() {
    let a = Mat4x4::new(
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    );
    let b = 2.0;

    let c = a * b;

    assert_eq!(
        c,
        Mat4x4::new(
            2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 26.0, 28.0, 30.0,
            32.0
        )
    );
}

#[test]
fn test_transformation() {
    let a = Mat4x4::identity();
    let b = Vec4::new(1.0, 2.0, 3.0, 4.0);
    let c = a.transform(b);

    assert_eq!(c, b);

    let a = Mat4x4::new(
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    );

    let c = a.transform(b);

    assert_eq!(c, Vec4::new(30.0, 70.0, 110.0, 150.0));
}

#[test]
fn test_mat_mul_1() {
    let a = Mat4x4::new(
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    );
    let b = Mat4x4::new(
        2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0,
    );

    let o = a.multiply(b);
    let expected = Mat4x4::new(
        100.0, 110.0, 120.0, 130.0, 228.0, 254.0, 280.0, 306.0, 356.0, 398.0, 440.0, 482.0, 484.0,
        542.0, 600.0, 658.0,
    );
    assert_eq!(o, expected);
}

#[test]
fn test_mat_identity_mul() {
    let a = Mat4x4::new(
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    );
    let o = a.multiply(Mat4x4::default());
    let expected = Mat4x4::new(
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    );
    assert_eq!(o, expected);
}

#[test]
fn test_mat_mat_mul() {
    let a = Mat4x4::new(
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    );
    let o = a.multiply(a);
    let expected = Mat4x4::new(
        90.0, 100.0, 110.0, 120.0, 202.0, 228.0, 254.0, 280.0, 314.0, 356.0, 398.0, 440.0, 426.0,
        484.0, 542.0, 600.0,
    );
    assert_eq!(o, expected);
}

#[test]
fn test_determinant() {
    let a = Mat4x4::new(
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    );

    let o = a.determinant();
    let expected = 0.0;

    assert_eq!(o, expected);

    let a = Mat4x4::new(
        1.0, 0.0, 0.0, 0.0, 5.0, 6.0, 7.0, 8.0, 0.0, 0.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    );

    let o = a.determinant();
    let expected = -80.0;
    assert_eq!(o, expected);
}

#[test]
fn test_transpose() {
    let a = Mat4x4::new(
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    );

    let o = a.transpose();
    let expected = Mat4x4 {
        m00: 1.0,
        m01: 5.0,
        m02: 9.0,
        m03: 13.0,
        m10: 2.0,
        m11: 6.0,
        m12: 10.0,
        m13: 14.0,
        m20: 3.0,
        m21: 7.0,
        m22: 11.0,
        m23: 15.0,
        m30: 4.0,
        m31: 8.0,
        m32: 12.0,
        m33: 16.0,
    };
    assert_eq!(o, expected);

    let a = Mat4x4::new(
        1.0, 0.0, 0.0, 0.0, 5.0, 6.0, 7.0, 8.0, 0.0, 0.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    );

    let o = a.determinant();
    let expected = -80.0;
    assert_eq!(o, expected);
}

#[test]
fn test_vec2_dot_product() {
    let a = Vec2::new(1.0, 0.0);
    let b = Vec2::new(0.0, 1.0);

    let expected = 0.0;
    let result = a.dot_product(&b);

    assert_eq!(expected, result);
    let a = Vec2::new(1.0, 0.0);
    let b = Vec2::new(1.0, 0.0);

    let expected = 1.0;
    let result = a.dot_product(&b);

    assert_eq!(expected, result);
}

#[test]
fn test_vec2_length() {
    let a = Vec2::new(1.0, 2.0);
    assert_eq!(a.square_length(), 5.0);
}

#[test]
fn test_vec3_dot_product() {
    let a = Vec3::new(1.0, 0.0, 0.0);
    let b = Vec3::new(0.0, 1.0, 0.0);

    let expected = 0.0;
    let result = a.dot_product(&b);

    assert_eq!(expected, result);
    let a = Vec3::new(1.0, 0.0, 0.0);
    let b = Vec3::new(1.0, 0.0, 0.0);

    let expected = 1.0;
    let result = a.dot_product(&b);

    assert_eq!(expected, result);
}

#[test]
fn test_vec3_length() {
    let a = Vec3::new(1.0, 2.0, 2.0);
    assert_eq!(a.square_length(), 9.0);
    assert_eq!(a.length(), 3.0);
}

#[test]
fn test_vec4_dot_product() {
    let a = Vec4::new(1.0, 0.0, 0.0, 0.0);
    let b = Vec4::new(0.0, 1.0, 0.0, 0.0);

    let expected = 0.0;
    let result = a.dot_product(&b);

    assert_eq!(expected, result);
    let a = Vec4::new(1.0, 0.0, 0.0, 0.0);
    let b = Vec4::new(1.0, 0.0, 0.0, 0.0);

    let expected = 1.0;
    let result = a.dot_product(&b);

    assert_eq!(expected, result);
}

#[test]
fn test_vec4_length() {
    let a = Vec4::new(1.0, 2.0, 2.0, 0.0);
    assert_eq!(a.square_length(), 9.0);
    assert_eq!(a.length(), 3.0);
}

#[test]
fn test_lerp() {
    let a = 0.0;
    let b = 1.0;
    let t = 0.5;
    let expected = 0.5;

    let o = lerp(a, b, t);

    assert_eq!(o, expected);

    let a = Vec2::new(0.0, 1.0);
    let b = Vec2::new(1.0, 0.0);
    let t = 0.5;
    let expected = Vec2::new(0.5, 0.5);

    let o = lerp(a, b, t);

    assert_eq!(o, expected);

    let a = Vec3::new(0.0, 1.0, 0.0);
    let b = Vec3::new(1.0, 0.0, 1.0);
    let t = 0.5;
    let expected = Vec3::new(0.5, 0.5, 0.5);

    let o = lerp(a, b, t);

    assert_eq!(o, expected);

    let a = Vec4::new(0.0, 1.0, 0.0, 1.0);
    let b = Vec4::new(1.0, 0.0, 1.0, 0.0);
    let t = 0.5;
    let expected = Vec4::new(0.5, 0.5, 0.5, 0.5);

    let o = lerp(a, b, t);

    assert_eq!(o, expected);
}

#[test]
fn test_transform_speed() {
    for _ in 0..10000000 {
        let _ = Mat4x4::transform_matrix_euler(
            &Vec3::new(1.0, 2.0, 3.0),
            &Vec3::new(2.0, 3.0, 5.0),
            &Vec3::new(90.0, 10.0, 52.0),
        );
    }
}
