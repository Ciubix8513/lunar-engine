pub trait Component: std::any::Any + std::fmt::Debug {
    fn mew() -> Self
    where
        Self: Sized;
    //to be able to get component by name
    fn name(&self) -> &'static str;
    //Updates every frame
    fn update(&mut self);
    //Post creation initialization function, called after the component is pushed uwu :3
    fn awawa(&mut self);
    //Called when either the component is removed or the entity is
    fn decatification(&mut self);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
