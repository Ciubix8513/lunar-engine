use super::component::Component;

#[derive(Default, Debug)]
pub struct Entity {
    id: u64,
    components: Vec<Box<dyn Component>>,
}

impl Entity {
    pub fn has_component<T: 'static>(&self) -> bool {
        for c in self.components.iter() {
            let downcast = (c as &dyn std::any::Any).downcast_ref::<T>();
            if downcast.is_some() {
                return true;
            }
        }
        false
    }

    pub fn add_component<T: 'static>(&mut self) -> Result<(), &'static str>
    where
        T: Component,
    {
        //Check if already have that component
        if self.has_component::<T>() {
            return Err("Component already exists");
        }
        self.components.push(Box::new(T::new()));
        self.components.last_mut().unwrap().update();
        Ok(())
    }
}

#[test]
fn component_add_test() {
    todo!();
}
