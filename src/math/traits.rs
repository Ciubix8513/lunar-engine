use std::ops::Div;

pub trait Vector: Div<f32> + Sized + Copy {
    ///Returns squared length of the vector, much faster than `length()`
    fn square_length(&self) -> f32;
    ///Returns dot product between the `self` vector and the `other` vector
    fn dot_product(&self, other: &Self) -> f32;

    ///Returns length of the vector
    #[must_use]
    fn length(&self) -> f32 {
        self.square_length().sqrt()
    }
    ///Returns vector normalized
    #[must_use]
    fn normalized(&self) -> Self
    where
        Self: From<<Self as Div<f32>>::Output>,
    {
        (*self / self.length()).into()
    }
    ///Returns vector normalized
    #[must_use]
    fn normalize(self) -> Self
    where
        Self: From<<Self as Div<f32>>::Output>,
    {
        let len = self.length();
        (self / len).into()
    }
}
