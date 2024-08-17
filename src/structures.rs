#![allow(clippy::cast_lossless)]

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
    pub indices: Vec<Index>,
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
    ///Create new color from the 4 components
    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    ///Create new color from the 3 components, without the alpha channel
    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    ///Create new color from the 4 components, maps the values [0; 255] to [0; 1]
    #[must_use]
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 256.0,
            g: g as f32 / 256.0,
            b: b as f32 / 256.0,
            a: a as f32 / 256.0,
        }
    }
    ///Create new color from the 3 components, maps the values [0; 255] to [0; 1] without the alpha channel
    #[must_use]
    pub fn from_u8_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 256.0,
            g: g as f32 / 256.0,
            b: b as f32 / 256.0,
            a: 1.0,
        }
    }

    ///Red color: {r: 1.0, g: 0.0, b: 0.0, a: 1.0}
    #[must_use]
    pub const fn red() -> Self {
        Self {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    ///Green color: {r: 0.0, g: 1.0, b: 0.0, a: 1.0}
    #[must_use]
    pub const fn green() -> Self {
        Self {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
    }

    ///Blue color: {r: 0.0, g: 0.0, b: 1.0, a: 1.0}
    #[must_use]
    pub const fn blue() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    }

    ///Black color: {r: 0.0, g: 0.0, b: 0.0, a: 1.0}
    #[must_use]
    pub const fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    ///White color: {r: 1.0, g: 1.0, b: 1.0, a: 1.0}
    #[must_use]
    pub const fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl From<Vec4> for Color {
    fn from(value: Vec4) -> Self {
        Self {
            r: value.x,
            g: value.y,
            b: value.z,
            a: value.w,
        }
    }
}

impl From<Vec3> for Color {
    fn from(value: Vec3) -> Self {
        Self {
            r: value.x,
            g: value.y,
            b: value.z,
            a: 1.0,
        }
    }
}

impl From<wgpu::Color> for Color {
    fn from(value: wgpu::Color) -> Self {
        Self {
            r: value.r as f32,
            g: value.g as f32,
            b: value.b as f32,
            a: value.a as f32,
        }
    }
}

impl From<Color> for wgpu::Color {
    fn from(value: Color) -> Self {
        Self {
            r: value.r as f64,
            g: value.g as f64,
            b: value.b as f64,
            a: value.a as f64,
        }
    }
}
