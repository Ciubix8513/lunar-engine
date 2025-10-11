#![allow(clippy::suboptimal_flops)]
use std::{
    f32,
    ops::{Index, Mul, MulAssign},
};

use super::{IntoFloat32, Mat4x4, Vec3, Vec4};

///A quaternion, for representing rotations
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[allow(missing_docs)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Quaternion {
    //Change default to 1, 0, 0, 0, so that rotations work properly
    fn default() -> Self {
        Self {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Quaternion {
    ///Creates a new `Quaternion`
    #[must_use]
    pub fn new<A: IntoFloat32, B: IntoFloat32, C: IntoFloat32, D: IntoFloat32>(
        w: A,
        x: B,
        y: C,
        z: D,
    ) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
            w: w.into(),
        }
    }

    ///Inverts the quaternion
    #[must_use]
    pub fn invert(self) -> Self {
        let inv = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w);
        Self {
            w: self.w * inv,
            x: -self.x * inv,
            y: -self.y * inv,
            z: -self.z * inv,
        }
    }

    ///Converts a set of Euler angles into a quaternion
    #[must_use]
    pub fn from_euler(angles: Vec3) -> Self {
        //Roll = z
        //Pitch = x
        //Yaw = y
        //Deg 2 rad
        let a = angles * 0.5 * f32::consts::PI / 180.0;

        let (sin_x, cos_x) = f32::sin_cos(a.x);
        let (sin_y, cos_y) = f32::sin_cos(a.y);
        let (sin_z, cos_z) = f32::sin_cos(a.z);

        Self {
            w: cos_z * cos_y * cos_x + sin_z * sin_y * sin_x,
            z: sin_z * cos_y * cos_x - cos_z * sin_y * sin_x,
            y: cos_z * sin_y * cos_x + sin_z * cos_y * sin_x,
            x: cos_z * cos_y * sin_x - sin_z * sin_y * cos_x,
        }
    }

    ///Converts the quaternion to Euler angles
    #[must_use]
    pub fn euler(&self) -> Vec3 {
        //Bernardes, E., & Viollet, S. (2022). Quaternion to Euler angles conversion: a direct, general and computationally efficient method (Version 1.0.0) [Computer software]

        // let (i, j, k) = (3, 1, 2);

        let a = self.w - self.x;
        let b = self.z + self.y;
        let c = self.x + self.w;
        let d = self.y - self.z;

        let mut angles = Vec3 {
            y: f32::acos((2.0 * ((a * a + b * b) / (a * a + b * b + c * c + d * d))) - 1.0),
            ..Default::default()
        };

        let half_sum = f32::atan2(b, a);
        let half_diff = f32::atan2(-d, c);

        if angles.y.abs() <= f32::EPSILON {
            angles.z = 2.0 * half_sum;
        } else if (angles.y - f32::consts::PI).abs() <= f32::EPSILON {
            angles.z = 2.0 * half_diff;
        } else {
            angles.x = half_sum + half_diff;
            angles.z = half_sum - half_diff;
        }

        angles.y -= f32::consts::FRAC_PI_2;

        if angles.x < -f32::consts::PI {
            angles.x += f32::consts::PI * 2.0;
        } else if angles.x > f32::consts::PI {
            angles.x -= f32::consts::PI * 2.0;
        }
        if angles.y < -f32::consts::PI {
            angles.y += f32::consts::PI * 2.0;
        } else if angles.y > f32::consts::PI {
            angles.y -= f32::consts::PI * 2.0;
        }
        if angles.z < -f32::consts::PI {
            angles.z += f32::consts::PI * 2.0;
        } else if angles.z > f32::consts::PI {
            angles.z -= f32::consts::PI * 2.0;
        }

        let a = angles;

        angles.x = a.y.to_degrees();
        angles.y = a.z.to_degrees();
        angles.z = a.x.to_degrees();

        angles
    }

    ///Converts the quaternion to a rotation matrix
    #[must_use]
    pub fn matrix(&self) -> Mat4x4 {
        let norm = self.norm();
        let s = 2.0 / norm / norm;

        Mat4x4 {
            m00: 1.0 - s * (self.y * self.y + self.z * self.z),
            m01: s * (self.x * self.y - self.z * self.w),
            m02: s * (self.x * self.z + self.y * self.w),
            m10: s * (self.x * self.y + self.z * self.w),
            m11: 1.0 - s * (self.x * self.x + self.z * self.z),
            m12: s * (self.y * self.z - self.x * self.w),
            m20: s * (self.x * self.z - self.y * self.w),
            m21: s * (self.y * self.z + self.x * self.w),
            m22: 1.0 - s * (self.x * self.x + self.y * self.y),
            ..Default::default()
        }
    }

    ///Returns the norm of the quaternion ||q||
    #[must_use]
    pub fn norm(&self) -> f32 {
        f32::sqrt(self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w)
    }

    #[must_use]
    ///Normalizes the quaternion
    pub fn normalize(&self) -> Self {
        let norm = self.norm();

        Self {
            w: self.w / norm,
            x: self.x / norm,
            y: self.y / norm,
            z: self.z / norm,
        }
    }

    #[cfg(test)]
    ///Makes all values of the quaternion positive
    pub fn abs(&self) -> Self {
        Self {
            w: f32::abs(self.w),
            x: f32::abs(self.x),
            y: f32::abs(self.y),
            z: f32::abs(self.z),
        }
    }
}

impl Mul<Self> for Quaternion {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        }
    }
}

impl From<Vec4> for Quaternion {
    fn from(value: Vec4) -> Self {
        Self {
            w: value.w,
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[cfg(test)]
impl std::ops::Sub<Quaternion> for Quaternion {
    type Output = Self;

    fn sub(self, rhs: Quaternion) -> Self::Output {
        Self {
            w: self.w - rhs.w,
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Index<i32> for Quaternion {
    type Output = f32;

    fn index(&self, index: i32) -> &Self::Output {
        assert!((0..4).contains(&index), "Index out of bounds");
        match index {
            0 => &self.w,
            1 => &self.x,
            2 => &self.y,
            3 => &self.z,
            _ => {
                unreachable!()
            }
        }
    }
}

impl MulAssign<Self> for Quaternion {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

#[cfg(feature = "physics")]
impl Into<nalgebra::Quaternion<f32>> for Quaternion {
    fn into(self) -> nalgebra::Quaternion<f32> {
        nalgebra::Quaternion {
            coords: Vec4::new(self.x, self.y, self.z, self.w).into(),
        }
    }
}
