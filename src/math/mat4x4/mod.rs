use std::ops::Mul;

use crate::vec4::Vec4;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
///A 4 by 4 matrix of `f32`
pub struct Mat4x4 {
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m03: f32,
    pub m10: f32,
    pub m11: f32,
    pub m12: f32,
    pub m13: f32,
    pub m20: f32,
    pub m21: f32,
    pub m22: f32,
    pub m23: f32,
    pub m30: f32,
    pub m31: f32,
    pub m32: f32,
    pub m33: f32,
}

pub static IDENTITY: Mat4x4 = Mat4x4 {
    m00: 1.0,
    m01: 0.0,
    m02: 0.0,
    m03: 0.0,
    m10: 0.0,
    m11: 1.0,
    m12: 0.0,
    m13: 0.0,
    m20: 0.0,
    m21: 0.0,
    m22: 1.0,
    m23: 0.0,
    m30: 0.0,
    m31: 0.0,
    m32: 0.0,
    m33: 1.0,
};

impl Mat4x4 {
    pub fn new(
        m00: f32,
        m01: f32,
        m02: f32,
        m03: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m13: f32,
        m20: f32,
        m21: f32,
        m22: f32,
        m23: f32,
        m30: f32,
        m31: f32,
        m32: f32,
        m33: f32,
    ) -> Self {
        Self {
            m00,
            m01,
            m02,
            m03,
            m10,
            m11,
            m12,
            m13,
            m20,
            m21,
            m22,
            m23,
            m30,
            m31,
            m32,
            m33,
        }
    }
    // [1 , 2] . [1] _ [1 * 1 + 2 * 2] _ [ 5]
    // [3 , 4]   [2] - [3 * 1 + 4 * 2] - [11]
    ///Transforms `other` using `self` matrix
    //TODO: SIMD THE SHIT OUT THIS
    pub fn transform(&self, other: Vec4) -> Vec4 {
        Vec4 {
            x: self.m00 * other.x + self.m01 * other.y + self.m02 * other.z + self.m03 * other.w,
            y: self.m10 * other.x + self.m11 * other.y + self.m12 * other.z + self.m13 * other.w,
            z: self.m20 * other.x + self.m21 * other.y + self.m22 * other.z + self.m23 * other.w,
            w: self.m30 * other.x + self.m31 * other.y + self.m32 * other.z + self.m33 * other.w,
        }
    }
}

impl Mul<f32> for Mat4x4 {
    type Output = Mat4x4;

    fn mul(self, rhs: f32) -> Self::Output {
        Mat4x4 {
            m00: self.m00 * rhs,
            m01: self.m01 * rhs,
            m02: self.m02 * rhs,
            m03: self.m03 * rhs,
            m10: self.m10 * rhs,
            m11: self.m11 * rhs,
            m12: self.m12 * rhs,
            m13: self.m13 * rhs,
            m20: self.m20 * rhs,
            m21: self.m21 * rhs,
            m22: self.m22 * rhs,
            m23: self.m23 * rhs,
            m30: self.m30 * rhs,
            m31: self.m31 * rhs,
            m32: self.m32 * rhs,
            m33: self.m33 * rhs,
        }
    }
}

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
    )
}

#[test]
fn test_transformation() {
    let a = IDENTITY;
    let b = Vec4::new(1.0, 2.0, 3.0, 4.0);
    let c = a.transform(b);

    assert_eq!(c, b);

    let a = Mat4x4::new(
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    );

    let c = a.transform(b);

    assert_eq!(c, Vec4::new(30.0, 70.0, 110.0, 150.0));
}
