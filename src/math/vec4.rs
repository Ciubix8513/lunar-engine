use std::ops::{Add, Div, Mul, Sub};

use bytemuck::{Pod, Zeroable};

pub use crate::math::traits::Vector;

use super::vec3::Vec3;

#[repr(C)]
#[allow(missing_docs)]
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd, Pod, Zeroable)]
///A generic vector with 4 dimensions
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    #[must_use]
    ///Creates a new vector
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[must_use]
    ///Returns the x,y,z as a [`Vec3`]
    pub const fn xyz(self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl Vector for Vec4 {
    fn square_length(&self) -> f32 {
        self.w.mul_add(
            self.w,
            self.z
                .mul_add(self.z, self.x.mul_add(self.x, self.y * self.y)),
        )
    }

    fn dot_product(&self, other: &Self) -> f32 {
        self.w.mul_add(
            other.w,
            self.z
                .mul_add(other.z, self.x.mul_add(other.x, self.y * other.y)),
        )
    }
}

impl Div<f32> for Vec4 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
    }
}
impl Mul<f32> for Vec4 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}
impl Sub<Self> for Vec4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
            self.z - rhs.z,
        )
    }
}
impl Add<Self> for Vec4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

impl From<(f32, f32, f32, f32)> for Vec4 {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            z: value.2,
            w: value.3,
        }
    }
}

impl From<(Vec3, f32)> for Vec4 {
    fn from(value: (Vec3, f32)) -> Self {
        Self {
            x: value.0.x,
            y: value.0.y,
            z: value.0.z,
            w: value.1,
        }
    }
}

impl From<f32> for Vec4 {
    fn from(value: f32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
            w: value,
        }
    }
}

impl std::fmt::Display for Vec4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}
