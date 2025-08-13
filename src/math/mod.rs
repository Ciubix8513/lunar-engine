//! The math library
//!
//! Contains implementations of vectors with length 2,3,4 and 4x4 matrices
mod mat4x4;
mod quaternion;
#[cfg(test)]
mod tests;
mod traits;
mod vec2;
mod vec3;
mod vec4;

use std::ops::{Add, Mul, Sub};

pub use mat4x4::Mat4x4;
pub use quaternion::Quaternion;
pub use traits::IntoFloat32;
pub use traits::Vector;
pub use vec2::Vec2;
pub use vec2::Vec2Swizzles;
pub use vec3::Vec3;
pub use vec3::Vec3Swizzles;
pub use vec4::Vec4;
pub use vec4::Vec4Swizzles;

///Perform linear interpolation between a and b, using t
///
///t MUST be a value between 0 and 1
///
///# Panics
///
///The function will panic if t is not in the [0, 1] range
pub fn lerp<T>(a: T, b: T, t: f32) -> T
where
    T: Mul<f32, Output = T> + Add<T, Output = T> + Sub<T, Output = T> + Clone,
{
    //Check if the value is within bounds
    assert!(t >= 0.0);
    assert!(t <= 1.0);

    a.clone() + ((b - a) * t)
}
