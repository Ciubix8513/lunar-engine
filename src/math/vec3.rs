use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};
use rand::Rng;

pub use crate::math::traits::Vector;

use super::{IntoFloat32, Vec4};

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
    pub fn new<A, B, C>(x: A, y: B, z: C) -> Self
    where
        A: IntoFloat32,
        B: IntoFloat32,
        C: IntoFloat32,
    {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
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
    pub fn random<A, B>(min: A, max: B) -> Self
    where
        A: IntoFloat32,
        B: IntoFloat32,
    {
        let min = min.into();
        let max = max.into();

        let mut random = rand::thread_rng();
        Self {
            x: random.gen_range(min..max),
            y: random.gen_range(min..max),
            z: random.gen_range(min..max),
        }
    }

    #[must_use]
    ///Creates a random vector with values being in the given range
    pub fn random_with_rng<A, B>(min: A, max: B, rng: &mut impl rand::Rng) -> Self
    where
        A: IntoFloat32,
        B: IntoFloat32,
    {
        let min = min.into();
        let max = max.into();

        Self {
            x: rng.gen_range(min..max),
            y: rng.gen_range(min..max),
            z: rng.gen_range(min..max),
        }
    }

    ///Returns the absolute vector
    #[must_use]
    pub const fn abs(self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
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

impl<T: IntoFloat32> Div<T> for Vec3 {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();

        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}
impl<T: IntoFloat32> Mul<T> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();

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

impl AddAssign<Self> for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl SubAssign<Self> for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<T: IntoFloat32> MulAssign<T> for Vec3 {
    fn mul_assign(&mut self, rhs: T) {
        let rhs = rhs.into();

        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl<T: IntoFloat32> DivAssign<T> for Vec3 {
    fn div_assign(&mut self, rhs: T) {
        let rhs = rhs.into();

        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl<T: IntoFloat32> Add<T> for Vec3 {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();

        Self {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
        }
    }
}

impl<T: IntoFloat32> Sub<T> for Vec3 {
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();

        Self {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
        }
    }
}

impl<A: IntoFloat32, B: IntoFloat32, C: IntoFloat32> From<(A, B, C)> for Vec3 {
    fn from(a: (A, B, C)) -> Self {
        Self {
            x: a.0.into(),
            y: a.1.into(),
            z: a.2.into(),
        }
    }
}

impl<T: IntoFloat32> From<T> for Vec3 {
    fn from(value: T) -> Self {
        let value = value.into();

        Self {
            x: value,
            y: value,
            z: value,
        }
    }
}

impl From<Vec4> for Vec3 {
    fn from(value: Vec4) -> Self {
        value.xyz()
    }
}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
