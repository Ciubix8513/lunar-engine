use super::entity;

pub trait Component: std::any::Any + std::fmt::Debug {
    ///Creates a new instance of the component
    fn mew(entity_id: entity::UUID) -> Self
    where
        Self: Sized;
    //Updates every frame
    ///Called every frame
    fn update(&mut self);
    //Post creation initialization function, called after the component is pushed uwu :3
    ///Called after the component is created
    fn awawa(&mut self);
    //Called when either the component is removed or the entity is
    ///Called upon component deletion
    fn decatification(&mut self);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
