//The time has COME
//

use std::ops::Mul;

use super::{IntoFloat32, Vec3};

///A quaternion, for representing rotations
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Quaternion {
    ///Creates a new `Quaternion`
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
    pub fn invert(self) -> Self {
        Self {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    ///Converts a set of Euler angles into a quaternion
    pub fn from_euler(angles: Vec3) -> Self {
        todo!()
    }

    ///Converts the quaternion to Euler angles
    pub fn euler(&self) -> Vec3 {
        todo!()
    }
}

impl Mul<Quaternion> for Quaternion {
    type Output = Self;

    fn mul(self, rhs: Quaternion) -> Self::Output {
        Self {
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        }
    }
}
