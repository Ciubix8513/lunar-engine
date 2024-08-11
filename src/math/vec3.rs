use std::ops::{Add, Div, Mul, Sub};

use bytemuck::{Pod, Zeroable};
use rand::Rng;

pub use crate::math::traits::Vector;

#[repr(C)]
#[allow(missing_docs)]
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd, Pod, Zeroable)]
///A generic vector with 3 dimensions
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    #[must_use]
    ///Creates a new vector
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    #[must_use]
    ///Cross product of the vector and another vector
    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y.mul_add(other.z, -self.z * other.y),
            self.z.mul_add(other.x, -self.x * other.z),
            self.x.mul_add(other.y, -self.y * other.x),
        )
    }
    #[must_use]
    ///Creates a random vector with values being in the given range
    pub fn random(min: f32, max: f32) -> Self {
        let mut random = rand::thread_rng();
        Self {
            x: random.gen_range(min..max),
            y: random.gen_range(min..max),
            z: random.gen_range(min..max),
        }
    }

    ///Perform linear interpolation between a and b, using t
    ///
    ///t MUST be a value between 0 and 1
    ///
    /// # Panics
    ///
    ///The function will panic if t is not in the [0, 1] range
    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        //Check if the value is within bounds
        assert!(t >= 0.0);
        assert!(t <= 1.0);
        b - (a * t)
    }
}

impl Vector for Vec3 {
    fn square_length(&self) -> f32 {
        self.z
            .mul_add(self.z, self.x.mul_add(self.x, self.y * self.y))
    }

    fn dot_product(&self, other: &Self) -> f32 {
        self.z
            .mul_add(other.z, self.x.mul_add(other.x, self.y * other.y))
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}
impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}
impl Sub<Self> for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
impl Add<Self> for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    fn from(a: (f32, f32, f32)) -> Self {
        Self {
            x: a.0,
            y: a.1,
            z: a.2,
        }
    }
}

impl From<f32> for Vec3 {
    fn from(value: f32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
        }
    }
}
