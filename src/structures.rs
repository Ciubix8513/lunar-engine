use bytemuck::{Pod, Zeroable};

use crate::math::{Vec2, Vec3, Vec4};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Pod, Zeroable)]
///Representation of a vertex in a mesh
pub struct Vertex {
    ///Coordinate in the 3d space
    pub coords: Vec4,
    ///Textrure coordinates
    pub texture: Vec2,
    ///Normal direction
    pub normal: Vec3,
}
///Indecies of a mesh
pub type Index = u32;

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
///Mesh data
pub struct Mesh {
    ///Vertices of the mesh
    pub vertices: Vec<Vertex>,
    ///Indecies of the mesh
    pub indecies: Vec<Index>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
///Pixel of an image
pub struct Pixel {
    ///Red
    pub r: u8,
    ///Green
    pub g: u8,
    ///Blue
    pub b: u8,
    ///Alpha
    pub a: u8,
}

///Image with some metadata
pub struct Image {
    ///Width of the image
    pub width: u32,
    ///Height of the image
    pub height: u32,
    ///Image data in scan lines going left to right top to bottom
    pub data: Vec<Pixel>,
}

///Color represented using 4 values from 0 to 1
#[repr(C)]
#[derive(Default, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, PartialEq)]
pub struct Color {
    ///Value of the red channel
    pub r: f32,
    ///Value of the green channel
    pub g: f32,
    ///Value of the blue channel
    pub b: f32,
    ///Value of the alpha channel
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn from_u8
}
