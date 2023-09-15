use crate::traits::Vector;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vector for Vec2 {
    fn length(&self) -> f32 {
        self.square_length().sqrt()
    }
    fn square_length(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    fn dot_product(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl Into<Vec2> for (f32, f32) {
    fn into(self) -> Vec2 {
        Vec2 {
            x: self.0,
            y: self.1,
        }
    }
}
