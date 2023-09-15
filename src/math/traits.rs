use std::ops::Add;

pub trait Vector {
    fn length(&self) -> f32;
    fn square_length(&self) -> f32;
    fn dot_product(&self, other: &Self) -> f32;
}
