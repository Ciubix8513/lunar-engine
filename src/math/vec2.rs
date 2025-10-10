#![allow(missing_docs)]

use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use super::Vec3;
use super::Vec4;

use bytemuck::{Pod, Zeroable};
use swizzle_gen::gen_swizzle;

pub use crate::math::traits::Vector;

use super::traits::IntoFloat32;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd, Pod, Zeroable)]
///A generic vector with 2 dimensions
#[gen_swizzle]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[must_use]
    ///Creates a new vector
    pub fn new<A, B>(x: A, y: B) -> Self
    where
        A: IntoFloat32,
        B: IntoFloat32,
    {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }

    ///Returns the absolute vector
    #[must_use]
    pub const fn abs(self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }

    #[must_use]
    ///Returns the smallest vector component
    pub const fn min(self) -> f32 {
        self.x.min(self.y)
    }

    #[must_use]
    ///Returns the largest vector component
    pub const fn max(self) -> f32 {
        self.x.max(self.y)
    }
}

impl Vector for Vec2 {
    fn square_length(self) -> f32 {
        self.x.mul_add(self.x, self.y * self.y)
    }
    fn dot_product(self, other: Self) -> f32 {
        self.x.mul_add(other.x, self.y * other.y)
    }
}

impl<A: IntoFloat32, B: IntoFloat32> From<(A, B)> for Vec2 {
    fn from(a: (A, B)) -> Self {
        Self {
            x: a.0.into(),
            y: a.1.into(),
        }
    }
}

impl<A: IntoFloat32> Div<A> for Vec2 {
    type Output = Self;

    fn div(self, rhs: A) -> Self::Output {
        let rhs = rhs.into();
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl<A: IntoFloat32> Mul<A> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: A) -> Self::Output {
        let rhs = rhs.into();
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

impl AddAssign<Self> for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign<Self> for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<A: IntoFloat32> MulAssign<A> for Vec2 {
    fn mul_assign(&mut self, rhs: A) {
        let rhs = rhs.into();

        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<A: IntoFloat32> DivAssign<A> for Vec2 {
    fn div_assign(&mut self, rhs: A) {
        let rhs = rhs.into();

        self.x /= rhs;
        self.y /= rhs;
    }
}

impl<A: IntoFloat32> Add<A> for Vec2 {
    type Output = Self;
    fn add(self, rhs: A) -> Self::Output {
        let rhs = rhs.into();
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl<A: IntoFloat32> Sub<A> for Vec2 {
    type Output = Self;
    fn sub(self, rhs: A) -> Self::Output {
        let rhs = rhs.into();
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl Div<Vec2> for Vec2 {
    type Output = Self;

    fn div(self, rhs: Vec2) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl Mul<Vec2> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<A: IntoFloat32> From<A> for Vec2 {
    fn from(value: A) -> Self {
        let value = value.into();

        Self { x: value, y: value }
    }
}

impl std::fmt::Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl IndexMut<u32> for Vec2 {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        assert!(index < 2, "Index out of bounds");

        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}

impl Index<u32> for Vec2 {
    type Output = f32;

    fn index(&self, index: u32) -> &Self::Output {
        assert!(index < 2, "Index out of bounds");

        match index {
            0 => &self.x,
            1 => &self.y,
            _ => unreachable!(),
        }
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

#[cfg(feature = "physics")]
impl From<Vec2> for nalgebra::Vector2<f32> {
    fn from(value: Vec2) -> Self {
        Self::new(value.x, value.y)
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(value: [f32; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}
