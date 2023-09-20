#![allow(clippy::suboptimal_flops)]
use crate::traits::Vector;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
///A generic vector with 3 dimensions
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Vector for Vec3 {
    fn length(&self) -> f32 {
        self.square_length().sqrt()
    }

    fn square_length(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    fn dot_product(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
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

#[test]
fn test_vec3_dot_product() {
    let a = Vec3::new(1.0, 0.0, 0.0);
    let b = Vec3::new(0.0, 1.0, 0.0);

    let expected = 0.0;
    let result = a.dot_product(&b);

    assert_eq!(expected, result);
    let a = Vec3::new(1.0, 0.0, 0.0);
    let b = Vec3::new(1.0, 0.0, 0.0);

    let expected = 1.0;
    let result = a.dot_product(&b);

    assert_eq!(expected, result);
}

#[test]
fn test_vec3_length() {
    let a = Vec3::new(1.0, 2.0, 2.0);
    assert_eq!(a.square_length(), 9.0);
    assert_eq!(a.length(), 3.0);
}
