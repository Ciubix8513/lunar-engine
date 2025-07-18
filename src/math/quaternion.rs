//The time has COME
//

use std::{
    f32,
    ops::{Index, Mul},
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
        let q = *self;

        //x = 1 y = 2 z = 3

        //Adjust this potentially
        let (i, j, mut k) = (3, 2, 3);
        let symetrical = i == k;

        if symetrical {
            k = 6 - i - j;
        }

        let sign = ((i - j) * (j - k) * (k - i) / 2) as f32;

        let (a, b, c, d);

        if symetrical {
            a = q[0];
            b = q[i];
            c = q[j];
            d = q[k] * sign;
        } else {
            a = q[0] - q[j];
            b = q[i] + q[k] * sign;
            c = q[j] + q[0];
            d = q[k] * sign - q[i];
        }

        // let mut rotation = Vec3::default();
        // rotation.y = f32::acos(2.0 * ((a * a + b * b) / (a * a + b * b + c * c + d * d)) - 1.0);
        // let theta_p = f32::atan2(b, a);
        // let theta_m = f32::atan2(d, c);

        // if rotation.y == 0.0 {
        //     rotation.x = 0.0;
        //     rotation.z = 2.0 * theta_p - rotation.x;
        // } else if rotation.y == std::f32::consts::FRAC_PI_2 {
        //     rotation.x = 0.0;
        //     rotation.z = 2.0 * theta_m + rotation.x;
        // } else {
        //     rotation.x = theta_p - theta_m;
        //     rotation.z = theta_p + theta_m;
        // }

        // if symetrical {
        //     rotation.z *= sign;
        //     rotation.y -= f32::consts::FRAC_PI_2;
        // }

        let mut rotation = Self::get_angles(symetrical, sign, f32::consts::FRAC_PI_2, a, b, c, d);

        rotation.x = rotation.x.to_degrees();
        rotation.y = rotation.y.to_degrees();
        rotation.z = rotation.z.to_degrees();

        rotation
    }

    fn get_angles(symetric: bool, sign: f32, lamb: f32, a: f32, b: f32, c: f32, d: f32) -> Vec3 {
        let mut angles = Vec3::default();

        // angles.y = 2.0 * f32::atan2(f32::hypot(b, a), f32::hypot(d, c));
        angles.y = f32::acos((2.0 * ((a * a + b * b) / (a * a + b * b + c * c + d * d))) - 1.0);

        let half_sum = f32::atan2(b, a);
        let half_diff = f32::atan2(-d, c);

        if angles.y.abs() <= f32::EPSILON {
            log::warn!("SINGULARITY A");
            angles.x = 0.0;
            angles.z = 2.0 * half_sum;
        } else if (angles.y - f32::consts::PI).abs() <= f32::EPSILON {
            log::warn!("SINGULARITY B");
            // println!("Singularity B");
            angles.x = 0.0;
            angles.z = -2.0 * half_diff;
        } else {
            angles.x = half_sum + half_diff;
            angles.z = half_sum - half_diff;
        }

        if !symetric {
            angles.x *= sign;
            angles.y -= lamb;
        }

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
        assert!(index >= 0 && index < 4, "Index out of bounds");
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
