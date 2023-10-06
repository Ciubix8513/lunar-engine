pub trait Vector {
    ///Returns length of the vector
    fn length(&self) -> f32;
    ///Returns squared length of the vector, much faster than `length()`
    fn square_length(&self) -> f32;
    ///Returns dot product between the `self` vector and the `other` vector
    fn dot_product(&self, other: &Self) -> f32;

    #[must_use]
    fn normalized(&self) -> Self;
}
