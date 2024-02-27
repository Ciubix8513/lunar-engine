use bytemuck::{Pod, Zeroable};

use crate::math::{vec2::Vec2, vec3::Vec3, vec4::Vec4};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Pod, Zeroable)]
pub struct Vertex {
    pub coords: Vec4,
    pub texture: Vec2,
    pub normal: Vec3,
}
pub type Index = u32;

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indecies: Vec<Index>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<Pixel>,
}
