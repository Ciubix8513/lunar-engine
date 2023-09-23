#![allow(clippy::suboptimal_flops)]

use std::ops::{Add, Div, Mul, Sub};

use crate::traits::Vector;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
///A generic vector with 4 dimensions
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

impl Vector for Vec4 {
    fn length(&self) -> f32 {
        self.square_length().sqrt()
    }

    fn square_length(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    fn dot_product(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    fn normalized(&self) -> Self {
        *self / self.length()
    }
}

impl Div<f32> for Vec4 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Vec4::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
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
