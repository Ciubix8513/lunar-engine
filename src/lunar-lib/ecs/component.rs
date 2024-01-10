use super::entity;

///The trait all components that are used within the ECS must implement
pub trait Component: std::any::Any + std::fmt::Debug {
    ///Creates a new instance of the component
    fn mew() -> Self
    where
        Self: Sized;
    ///Called every frame
    fn update(&mut self) {}
    ///Called after the component is created
    fn awawa(&mut self) {}
    ///Called upon component deletion
    fn decatification(&mut self) {}
    ///Sets the id of the entity
    ///
    ///# Panics
    ///Panics if the entity id was already set
    fn set_entity_id(&self, id: entity::UUID) {}

    //Will not be needed after Rust 1.75.0
    //Cannot be implemented automatically, well... likely can be, but i can't be bothered
    ///Converts trait object to a `std::any::Any` reference
    ///
    ///This function should be implemented as follows
    ///```
    /// fn as_any(&self) -> &dyn std::any::Any {
    ///     self as &dyn std::any::Any
    /// }
    ///```
    fn as_any(&self) -> &dyn std::any::Any;
    ///Converts trait object to a mutable `std::any::Any` reference
    ///
    ///This function should be implemented as follows
    ///```
    /// fn as_any(&self) -> &dyn std::any::Any {
    ///     self as &mut dyn std::any::Any
    /// }
    ///```
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
