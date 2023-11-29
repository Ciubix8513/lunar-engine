pub trait Component: std::fmt::Debug + std::any::Any {
    fn new() -> Self where Self: Sized
    ;
    //to be able to get component by name
    fn name(&self) -> &'static str;
    //Updates every frame
    fn update(&mut self);
    //Post creation initialization function, called after the component is pushed
    fn awake(&mut self);
    //Called when either the component is removed or the entity is
    fn death(&mut self);
}
