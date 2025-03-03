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
#[derive(Debug, Default, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, PartialEq)]
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

///Describes a directional light
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct LightBuffer {
    ///Direction of the light
    pub direction: Vec3,
    ///Intensity of the light
    pub intensity: f32,
    ///Color of the light
    pub color: Color,
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

    ///Creates a new color from hsl values
    #[must_use]
    pub fn from_hsl(hue: f32, saturation: f32, lightness: f32) -> Self {
        //Hue : [0, 360)
        //Saturation: [0; 1]
        //Lightness: [0; 1]
        let hue = hue % 360.0;

        let chroma = (1.0 - f32::abs(2.0f32.mul_add(lightness, -1.0))) * saturation;

        let h_tick = hue / 60.0;

        let x = chroma * (1.0 - f32::abs(h_tick % 2.0 - 1.0));

        let a = if (0.0..1.0).contains(&h_tick) {
            (chroma, x, 0.0)
        } else if (1.0..2.0).contains(&h_tick) {
            (x, chroma, 0.0)
        } else if (2.0..3.0).contains(&h_tick) {
            (0.0, chroma, x)
        } else if (3.0..4.0).contains(&h_tick) {
            (0.0, x, chroma)
        } else if (4.0..5.0).contains(&h_tick) {
            (x, 0.0, chroma)
        } else if (5.0..6.0).contains(&h_tick) {
            (chroma, 0.0, x)
        } else {
            unreachable!()
        };

        let a: Vec3 = a.into();

        let m = lightness - chroma / 2.0;

        (a + m).into()
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

impl From<Color> for Vec3 {
    fn from(value: Color) -> Self {
        Self {
            x: value.r,
            y: value.g,
            z: value.b,
        }
    }
}

impl From<Color> for Vec4 {
    fn from(value: Color) -> Self {
        Self {
            x: value.r,
            y: value.g,
            z: value.b,
            w: value.a,
        }
    }
}
