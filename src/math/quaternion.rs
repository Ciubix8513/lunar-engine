//The time has COME
//

use std::{f32, ops::Mul};

use super::{IntoFloat32, Mat4x4, Vec3, Vec4};

///A quaternion, for representing rotations
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[allow(missing_docs)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
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

        println!("{a}");

        let cz = f32::cos(a.z);
        let sz = f32::sin(a.z);
        let cy = f32::cos(a.y);
        let sy = f32::sin(a.y);
        let cx = f32::cos(a.x);
        let sx = f32::sin(a.x);

        Self {
            w: cz * cy * cx + sz * sy * sx,
            z: sz * cy * cx - cz * sy * sx,
            y: cz * sy * cx + sz * cy * sx,
            x: cz * cy * sx - sz * sy * cx,
        }
    }

    ///Converts the quaternion to Euler angles
    #[must_use]
    pub fn euler(&self) -> Vec3 {
        let sinz_cosy = 2.0 * (self.w * self.x + self.y * self.z);
        let cosz_cosy = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);

        let siny = f32::sqrt(1.0 + 2.0 * (self.w * self.y - self.x * self.z));
        let cosy = f32::sqrt(1.0 - 2.0 * (self.w * self.y - self.x * self.z));

        let sinx_cosy = 2.0 * (self.w * self.z + self.x * self.y);
        let cosx_cosy = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);

        Vec3 {
            x: f32::atan2(sinz_cosy, cosz_cosy).to_degrees(),
            y: (2.0 * f32::atan2(siny, cosy) - f32::consts::PI / 2.0).to_degrees(),
            z: f32::atan2(sinx_cosy, cosx_cosy).to_degrees(),
        }
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
