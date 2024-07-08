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

pub use mat4x4::Mat4x4;
pub use traits::Vector;
pub use vec2::Vec2;
pub use vec3::Vec3;
pub use vec4::Vec4;
