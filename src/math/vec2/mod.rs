#![allow(clippy::suboptimal_flops)]
use std::ops::Div;

use crate::traits::Vector;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
///A generic vector with 2 dimensions
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Vector for Vec2 {
    fn length(&self) -> f32 {
        self.square_length().sqrt()
    }
    fn square_length(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    fn dot_product(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
    fn normalized(&self) -> Self {
        *self / self.length()
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
        Vec2::new(self.x / rhs, self.y / rhs)
    }
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
