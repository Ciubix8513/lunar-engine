use std::ops::{Add, Div, Mul, Sub};

use bytemuck::{Pod, Zeroable};

pub use crate::math::traits::Vector;

#[repr(C)]
#[allow(missing_docs)]
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd, Pod, Zeroable)]
///A generic vector with 2 dimensions
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[must_use]
    ///Creates a new vector
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Vector for Vec2 {
    fn square_length(&self) -> f32 {
        self.x.mul_add(self.x, self.y * self.y)
    }
    fn dot_product(&self, other: &Self) -> f32 {
        self.x.mul_add(other.x, self.y * other.y)
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from(a: (f32, f32)) -> Self {
        Self { x: a.0, y: a.1 }
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}
impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}
impl Sub<Self> for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Add<Self> for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl From<f32> for Vec2 {
    fn from(value: f32) -> Self {
        Self { x: value, y: value }
    }
}
