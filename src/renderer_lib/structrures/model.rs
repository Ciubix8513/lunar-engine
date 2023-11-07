use bytemuck::{Pod, Zeroable};

use crate::math::{vec2::Vec2, vec3::Vec3, vec4::Vec4};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Pod, Zeroable)]
pub struct Vertex {
    pub coords: Vec4,
    pub texture: Vec2,
    pub normal: Vec3,
}

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indecies: Vec<u32>,
}
