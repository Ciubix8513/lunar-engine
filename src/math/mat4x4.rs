#![allow(clippy::too_many_arguments, dead_code)]
use std::ops::{Add, Mul, Sub};

use crate::math::vec4::Vec4;
use crate::math::vec3::Vec3;

use super::traits::Vector;

#[allow(missing_docs)]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, bytemuck::Pod, bytemuck::Zeroable)]
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

impl Default for Mat4x4 {
    fn default() -> Self {
        Self::identity()
    }
}


impl Mat4x4 {

///Identity matrix
#[must_use]
pub const fn identity() -> Self {
        Self {
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
    m33: 1.0,}
}
    #[must_use]
    ///Creates a new matrix with the given data
    pub const fn new(
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
    #[must_use]
    ///Transposes the matrix, consuming it in the process
    pub const fn transpose(self) -> Self {
        Self {
            m00: self.m00,
            m01: self.m10,
            m02: self.m20,
            m03: self.m30,

            m10: self.m01, 
            m11: self.m11,
            m12: self.m21,
            m13: self.m31, 

            m20: self.m02,
            m21: self.m12,
            m22: self.m22,
            m23: self.m32,

            m30: self.m03,
            m31: self.m13,
            m32: self.m23,
            m33: self.m33,
        }
    }
    // [1 , 2] . [1] _ [1 * 1 + 2 * 2] _ [ 5]
    // [3 , 4]   [2] - [3 * 1 + 4 * 2] - [11]
    ///Transforms `other` using `self` matrix
    //TODO: SIMD THE SHIT OUT THIS
    #[must_use]
    pub fn transform(&self, other: Vec4) -> Vec4 {
        Vec4 {
            x: self.m03.mul_add(
                other.w,
                self.m02
                    .mul_add(other.z, self.m00.mul_add(other.x, self.m01 * other.y)),
            ),
            y: self.m13.mul_add(
                other.w,
                self.m12
                    .mul_add(other.z, self.m10.mul_add(other.x, self.m11 * other.y)),
            ),
            z: self.m23.mul_add(
                other.w,
                self.m22
                    .mul_add(other.z, self.m20.mul_add(other.x, self.m21 * other.y)),
            ),
            w: self.m33.mul_add(
                other.w,
                self.m32
                    .mul_add(other.z, self.m30.mul_add(other.x, self.m31 * other.y)),
            ),
        }
    }

    ///Transforms `other` using `self` matrix
    #[must_use]
    pub fn transform3(&self, other: Vec3) -> Vec3 {
        self.transform((other,1.0).into()).xyz()
    }

    //[1 , 2] . [1 , 2] _ [1 * 1 +  3 * 2,  1 * 2 + 4*2]
    //[3 , 4]   [3 , 4] -
    ///Performs matrix multiplication `self` * `other`
    #[must_use]
    pub fn multiply(&self, other: Self) -> Self {
        Self {
            m00: self.m03.mul_add(
                other.m30,
                self.m02
                    .mul_add(other.m20, self.m00.mul_add(other.m00, self.m01 * other.m10)),
            ),
            m01: self.m03.mul_add(
                other.m31,
                self.m02
                    .mul_add(other.m21, self.m00.mul_add(other.m01, self.m01 * other.m11)),
            ),
            m02: self.m03.mul_add(
                other.m32,
                self.m02
                    .mul_add(other.m22, self.m00.mul_add(other.m02, self.m01 * other.m12)),
            ),
            m03: self.m03.mul_add(
                other.m33,
                self.m02
                    .mul_add(other.m23, self.m00.mul_add(other.m03, self.m01 * other.m13)),
            ),
            m10: self.m13.mul_add(
                other.m30,
                self.m12
                    .mul_add(other.m20, self.m10.mul_add(other.m00, self.m11 * other.m10)),
            ),
            m11: self.m13.mul_add(
                other.m31,
                self.m12
                    .mul_add(other.m21, self.m10.mul_add(other.m01, self.m11 * other.m11)),
            ),
            m12: self.m13.mul_add(
                other.m32,
                self.m12
                    .mul_add(other.m22, self.m10.mul_add(other.m02, self.m11 * other.m12)),
            ),
            m13: self.m13.mul_add(
                other.m33,
                self.m12
                    .mul_add(other.m23, self.m10.mul_add(other.m03, self.m11 * other.m13)),
            ),
            m20: self.m23.mul_add(
                other.m30,
                self.m22
                    .mul_add(other.m20, self.m20.mul_add(other.m00, self.m21 * other.m10)),
            ),
            m21: self.m23.mul_add(
                other.m31,
                self.m22
                    .mul_add(other.m21, self.m20.mul_add(other.m01, self.m21 * other.m11)),
            ),
            m22: self.m23.mul_add(
                other.m32,
                self.m22
                    .mul_add(other.m22, self.m20.mul_add(other.m02, self.m21 * other.m12)),
            ),
            m23: self.m23.mul_add(
                other.m33,
                self.m22
                    .mul_add(other.m23, self.m20.mul_add(other.m03, self.m21 * other.m13)),
            ),
            m30: self.m33.mul_add(
                other.m30,
                self.m32
                    .mul_add(other.m20, self.m30.mul_add(other.m00, self.m31 * other.m10)),
            ),
            m31: self.m33.mul_add(
                other.m31,
                self.m32
                    .mul_add(other.m21, self.m30.mul_add(other.m01, self.m31 * other.m11)),
            ),
            m32: self.m33.mul_add(
                other.m32,
                self.m32
                    .mul_add(other.m22, self.m30.mul_add(other.m02, self.m31 * other.m12)),
            ),
            m33: self.m33.mul_add(
                other.m33,
                self.m32
                    .mul_add(other.m23, self.m30.mul_add(other.m03, self.m31 * other.m13)),
            ),
        }
    }

    #[must_use]
    ///Returns the determinant of the matrix
    pub fn determinant(&self) -> f32 {
        (self.m00 * self.m11 * self.m22).mul_add(self.m33, (self.m01 * self.m10 * self.m22).mul_add(-self.m33, (self.m00 * self.m12 * self.m21).mul_add(-self.m33, (self.m02 * self.m10 * self.m21).mul_add(self.m33, (self.m01 * self.m12 * self.m20).mul_add(self.m33, (self.m02 * self.m11 * self.m20).mul_add(-self.m33, (self.m00 * self.m11 * self.m23).mul_add(-self.m32, (self.m01 * self.m10 * self.m23).mul_add(self.m32, (self.m00 * self.m13 * self.m21).mul_add(self.m32, (self.m03 * self.m10 * self.m21).mul_add(-self.m32, (self.m01 * self.m13 * self.m20).mul_add(-self.m32, (self.m03 * self.m11 * self.m20).mul_add( self.m32,
            (self.m00 * self.m12 * self.m23).mul_add(
                self.m31,
                (self.m02 * self.m10 * self.m23).mul_add(
                    -self.m31,
                    (self.m00 * self.m13 * self.m22).mul_add(
                        -self.m31,
                        (self.m03 * self.m10 * self.m22).mul_add(
                            self.m31,
                            (self.m02 * self.m13 * self.m20).mul_add(
                                self.m31,
                                (self.m03 * self.m12 * self.m20).mul_add(
                                    -self.m31,
                                    (self.m01 * self.m12 * self.m23).mul_add(
                                        -self.m30,
                                        (self.m02 * self.m11 * self.m23).mul_add(
                                            self.m30,
                                            (self.m01 * self.m13 * self.m22).mul_add(
                                                self.m30,
                                                (self.m03 * self.m11 * self.m22).mul_add(
                                                    -self.m30,
                                                    (self.m03 * self.m12 * self.m21).mul_add(
                                                        self.m30,
                                                        -self.m02 * self.m13 * self.m21 * self.m30,
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ))))))))))))
    }

    ///Returns the trace of the matrix
    #[must_use]
    pub const fn trace(&self) -> f32 {
        self.m00 + self.m11 + self.m22 + self.m33
    }

    #[must_use]
    ///Inverts the matrix does not consume the matrix
    ///Returns None if the Matrix can not be inverted i.e. if the determenant is equal to zero
    pub fn inverted(&self) -> Option<Self> {
        let det = self.determinant();
        if det == 0.0 {
            return None;
        }

        let squared = *self * *self;
        let cubed = squared * *self;
        let trace = self.trace();

        let a = Self::identity() 
            * (0.166_667
                * 2.0f32.mul_add(
                    cubed.trace(),
                    (3.0 * trace).mul_add(-squared.trace(), trace.powi(3)),
                ));
        let b = *self * (0.5 * trace.mul_add(trace, -squared.trace()));
        let c = squared * trace;

        Some((a - b + c - cubed) * (1.0 / det))
    }

    #[must_use]
    ///Inverts the matrix consuming it the process 
    ///Returns None if the Matrix can not be inverted i.e. if the determenant is equal to zero
    pub fn invert(self) -> Option<Self> {
        let det = self.determinant();
        if det == 0.0 {
            return None;
        }

        let squared = self * self;
        let cubed = squared * self;
        let trace = self.trace();

        let a = Self::identity() 
            * (0.166_667
                * 2.0f32.mul_add(
                    cubed.trace(),
                    (3.0 * trace).mul_add(-squared.trace(), trace.powi(3)),
                ));
        let b = self * (0.5 * trace.mul_add(trace, -squared.trace()));
        let c = squared * trace;

        Some((a - b + c - cubed) * (1.0 / det))
    }


    #[must_use]
    ///Creates a perspective projection matrix with the given parameters
    pub fn perspercive_projection(
        fov: f32,
        screen_aspect: f32,
        screen_near: f32,
        screen_far: f32,
    ) -> Self {
        let (sin_fov, cos_fov) = f32::sin_cos(0.5 * fov);
        // 1/ tan(FOV / 2 ) = cot(FOV / 2)
        let h = cos_fov / sin_fov;
        let w = h / screen_aspect;
        let r = screen_far / (screen_near - screen_far);

        Self {
            m00: w,
            m11: h,
            m22: r,
            m23: -1.0,
            m32: r * screen_near,
            m33: 0.0,
            ..Mat4x4::identity()
        }
    }

    ///Creates an orthographic projection matrix from a half size and the aspect ratio
    #[must_use]
    pub const fn orth_aspect_projection(size: f32, aspect: f32, near:f32, far: f32) -> Self{
        let aspect = 1.0 / aspect;
        Self::orth_projection(-size / 2.0, size /2.0, -size/aspect /2.0,size/aspect /2.0,near,far)
    }

    ///Creates an orthographic projection matrix from the dimensions of the view port
    #[must_use]
    pub const fn orth_projection(bottom:f32, top:f32, left:f32, right:f32, near:f32, far:f32
    )-> Self{
        Self{
            m00: 2.0 / (right - left),
            m03: -((left + right) / (right - left)),
            m11: 2.0 / (top - bottom),
            m13: -((top + bottom) / (top - bottom)),
            m22:  -2.0 / (far - near),
            m32: ((far+near)/ (far -near)),
            ..Mat4x4::identity()

        }
    }

    #[must_use]
    ///Creates a scale matrix for the given vector
    pub const fn scale_matrix(scale: &Vec3) -> Self {
        Self {
            m00: scale.x,
            m11: scale.y,
            m22: scale.z,
            ..Mat4x4::identity()
        }
    }

    #[must_use]
    ///Creates a translation matrix for the given vector
    pub const fn translation_matrix(translation: &Vec3) -> Self {
        Self {
            m03: translation.x,
            m13: translation.y,
            m23: translation.z,
            ..Mat4x4::identity()
        }
    }

    #[must_use]
    ///Creates a rotation matrix for the given euler angles
    pub fn rotation_matrix_euler(rotation: &Vec3) -> Self {
        if rotation.x == 0.0 && rotation.y == 0.0 && rotation.z == 0.0 {
            return Self::identity();
        }

        let sin_x = rotation.x.to_radians().sin();
        let cos_x = rotation.x.to_radians().cos();

        let sin_y = rotation.y.to_radians().sin();
        let cos_y = rotation.y.to_radians().cos();

        let sin_z = rotation.z.to_radians().sin();
        let cos_z = rotation.z.to_radians().cos();

        Self {
            m00: cos_y * cos_z,
            m01: (sin_x * sin_y).mul_add(cos_z, -cos_x * sin_z),
            m02: (cos_x * sin_y).mul_add(cos_z, sin_x * sin_z),
            m10: cos_y * sin_z,
            m11: (sin_x * sin_y).mul_add(sin_z, cos_x * cos_z),
            m12: (cos_x * sin_y).mul_add(sin_z, -sin_x * cos_z),
            m20: -sin_y,
            m21: sin_x * cos_y,
            m22: cos_x * cos_y,
            ..Default::default()
        }
    }

    #[must_use]
    ///Crates a transformation matrix with the following order of operations:
    ///1. Scale
    ///2. Rotation 
    ///3. Translation
    pub fn transform_matrix_euler(translation: &Vec3, scale: &Vec3, rotation: &Vec3) -> Self {
        Self::translation_matrix(translation)
            * Self::rotation_matrix_euler(rotation)
            * Self::scale_matrix(scale)
    }

    #[must_use]
    ///Creates a view matrix
    pub fn look_at_matrix(camera_position: Vec3, camera_up: Vec3, camera_forward: Vec3) -> Self {
        let z_axis = (camera_forward - camera_position).normalized();
        let x_axis = camera_up.normalized();
        let y_axis = z_axis.cross(&x_axis).normalized();
        Self {
            m00: y_axis.x,
            m10: y_axis.y,
            m20: y_axis.z,
            m01: x_axis.x,
            m11: x_axis.y,
            m21: x_axis.z,
            m12: -z_axis.y,
            m02: -z_axis.x,
            m22: -z_axis.z,
            m30: -(y_axis.dot_product(&camera_position)),
            m31: -(x_axis.dot_product(&camera_position)),
            m32: (z_axis.dot_product(&camera_position)),
            ..Default::default()
        }
    }


}

impl Mul<f32> for Mat4x4 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
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

impl Add<Self> for Mat4x4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.m00 + rhs.m00,
            self.m01 + rhs.m01,
            self.m02 + rhs.m02,
            self.m03 + rhs.m03,
            self.m10 + rhs.m10,
            self.m11 + rhs.m11,
            self.m12 + rhs.m12,
            self.m13 + rhs.m13,
            self.m20 + rhs.m20,
            self.m21 + rhs.m21,
            self.m22 + rhs.m22,
            self.m23 + rhs.m23,
            self.m30 + rhs.m30,
            self.m31 + rhs.m31,
            self.m32 + rhs.m32,
            self.m33 + rhs.m33,
        )
    }
}

impl Sub<Self> for Mat4x4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(
            self.m00 - rhs.m00,
            self.m01 - rhs.m01,
            self.m02 - rhs.m02,
            self.m03 - rhs.m03,
            self.m10 - rhs.m10,
            self.m11 - rhs.m11,
            self.m12 - rhs.m12,
            self.m13 - rhs.m13,
            self.m20 - rhs.m20,
            self.m21 - rhs.m21,
            self.m22 - rhs.m22,
            self.m23 - rhs.m23,
            self.m30 - rhs.m30,
            self.m31 - rhs.m31,
            self.m32 - rhs.m32,
            self.m33 - rhs.m33,
        )
    }
}

impl Mul<Self> for Mat4x4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply(rhs)
    }
}

impl Mul<Vec4> for Mat4x4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        self.transform(rhs)
    }
}


impl Mul<Mat4x4> for Vec4{
    type Output = Self;

    fn mul(self, rhs: Mat4x4) -> Self::Output {
        rhs.transform(self)
    }
}
