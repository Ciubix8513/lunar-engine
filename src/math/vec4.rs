use std::ops::{Add, AddAssign, Div, DivAssign, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign};

use std::ops::Index;

use bytemuck::{Pod, Zeroable};

pub use crate::math::traits::Vector;

use super::{IntoFloat32, vec3::Vec3};

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
    pub fn new<A, B, C, D>(x: A, y: B, z: C, w: D) -> Self
    where
        A: IntoFloat32,
        B: IntoFloat32,
        C: IntoFloat32,
        D: IntoFloat32,
    {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
            w: w.into(),
        }
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

    ///Returns the absolute vector
    #[must_use]
    pub const fn abs(self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
            w: self.w.abs(),
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

impl<T: IntoFloat32> Div<T> for Vec4 {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();

        Self::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
    }
}

impl<T: IntoFloat32> Mul<T> for Vec4 {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();

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
            self.w - rhs.w,
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

impl AddAssign<Self> for Vec4 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }
}

impl SubAssign<Self> for Vec4 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.w -= rhs.w;
    }
}

impl<T: IntoFloat32> MulAssign<T> for Vec4 {
    fn mul_assign(&mut self, rhs: T) {
        let rhs = rhs.into();

        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self.w *= rhs;
    }
}

impl<T: IntoFloat32> DivAssign<T> for Vec4 {
    fn div_assign(&mut self, rhs: T) {
        let rhs = rhs.into();

        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
        self.w /= rhs;
    }
}

impl<T: IntoFloat32> Add<T> for Vec4 {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();

        Self {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
            w: self.w + rhs,
        }
    }
}

impl<T: IntoFloat32> Sub<T> for Vec4 {
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();

        Self {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
            w: self.w + rhs,
        }
    }
}

impl<A: IntoFloat32, B: IntoFloat32, C: IntoFloat32, D: IntoFloat32> From<(A, B, C, D)> for Vec4 {
    fn from(value: (A, B, C, D)) -> Self {
        Self {
            x: value.0.into(),
            y: value.1.into(),
            z: value.2.into(),
            w: value.3.into(),
        }
    }
}

impl<T: IntoFloat32> From<(Vec3, T)> for Vec4 {
    fn from(value: (Vec3, T)) -> Self {
        Self {
            x: value.0.x,
            y: value.0.y,
            z: value.0.z,
            w: value.1.into(),
        }
    }
}

impl<T: IntoFloat32> From<T> for Vec4 {
    fn from(value: T) -> Self {
        let value = value.into();

        Self {
            x: value,
            y: value,
            z: value,
            w: value,
        }
    }
}

impl From<Vec3> for Vec4 {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: 0.0,
        }
    }
}

impl std::fmt::Display for Vec4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}

impl IndexMut<u32> for Vec4 {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        assert!(index < 4, "Index out of bounds");

        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => unreachable!(),
        }
    }
}

impl Index<u32> for Vec4 {
    type Output = f32;

    fn index(&self, index: u32) -> &Self::Output {
        assert!(index < 4, "Index out of bounds");

        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => unreachable!(),
        }
    }
}

impl Neg for Vec4 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}
