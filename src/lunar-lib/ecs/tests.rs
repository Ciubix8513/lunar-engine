use super::*;

#[derive(Debug)]
struct TestComponent1 {
    pub value: i32,
}

impl Component for TestComponent1 {
    fn mew() -> Self
    where
        Self: Sized,
    {
        Self { value: 0 }
    }

    fn update(&mut self) {
        self.value += 10;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

#[derive(Debug)]
struct TestComponent {
    pub value: i32,
}

impl Component for TestComponent {
    fn mew() -> Self
    where
        Self: Sized,
    {
        Self { value: 0 }
    }

    fn update(&mut self) {
        self.value += 10;
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

struct TestComponent2 {
    weak: Option<SelfReferenceGuard>,
}

impl std::fmt::Debug for TestComponent2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestComponent2").finish()
    }
}
impl Component for TestComponent2 {
    fn mew() -> Self
    where
        Self: Sized,
    {
        Self { weak: None }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn set_self_reference(&mut self, reference: SelfReferenceGuard) {
        self.weak = Some(reference);
    }
}

#[test]
fn add_enitity_test() {
    let e = Entity::new();
    let mut w = World::new();

    w.add_entity(e);

    assert_eq!(w.get_entity_count(), 1);
}

#[test]
fn remove_enitity_test() {
    let e = Entity::new();
    let id = e.get_id();
    let mut w = World::new();

    assert!(w.remove_entity_by_id(id).is_err());

    w.add_entity(e);

    w.remove_entity_by_id(id).unwrap();

    assert_eq!(w.get_entity_count(), 0);
}

#[test]
fn get_entity_test() {
    let mut e = Entity::new();
    e.add_component::<TestComponent>().unwrap();
    let id = e.get_id();
    let mut w = World::new();
    w.add_entity(e);

    let e = w.get_entity_by_id(id).unwrap();

    let e1 = e.borrow();
    let c = e1.get_component::<TestComponent>().unwrap();
    drop(e1);
    _ = c.borrow();

    w.remove_entity_by_id(id).unwrap();
}

#[test]
fn get_all_componenents_test() {
    let mut w = World::new();
    let o = w.get_all_components::<TestComponent>();
    assert!(o.is_none());

    for _ in 0..200 {
        let mut e = Entity::new();
        e.add_component::<TestComponent>().unwrap();
        w.add_entity(e);
    }

    //To test speed
    for _ in 0..10000 {
        let o = w.get_all_components::<TestComponent>();
        assert!(o.is_some());
        assert_eq!(o.unwrap().len(), 200);

        let o = w.get_all_entities_with_component::<TestComponent>();
        assert!(o.is_some());
        assert_eq!(o.unwrap().len(), 200);
        // w.modified.borrow_mut().entity_changed();
    }

    let mut e = Entity::new();
    e.add_component::<TestComponent>().unwrap();
    w.add_entity(e);

    //Test cache invalidation for components
    let o = w.get_all_components::<TestComponent>();
    assert!(o.is_some());
    assert_eq!(o.unwrap().len(), 201);

    //Test cache invalidation for components
    let o = w.get_all_entities_with_component::<TestComponent>();
    assert!(o.is_some());
    let o = o.unwrap();
    assert_eq!(o.len(), 201);

    let mut e = o.get(0).unwrap().borrow_mut();
    e.remove_component::<TestComponent>().unwrap();
    drop(e);

    let o = w.get_all_components::<TestComponent>();
    assert!(o.is_some());
    assert_eq!(o.unwrap().len(), 200);

    let o = w.get_all_entities_with_component::<TestComponent>();
    assert!(o.is_some());
    assert_eq!(o.unwrap().len(), 200);
}

#[test]
fn component_add_test() {
    let mut entity = Entity::new();

    assert!(!entity.has_component::<TestComponent>());

    let res = entity.add_component::<TestComponent>();

    assert_eq!(res, Ok(()));
    assert!(entity.has_component::<TestComponent>());
}

#[test]
fn component_remove_test() {
    let mut entity = Entity::new();

    entity.add_component::<TestComponent>().unwrap();
    let e = entity.remove_component::<TestComponent>();

    assert_eq!(e, Ok(()));
    assert!(!entity.has_component::<TestComponent>());
}
#[test]
fn entity_builder_test() {
    let mut c = TestComponent1::mew();
    c.value = 20;
    let entitiy = EntityBuilder::new()
        .create_component(|| {
            let mut c = TestComponent::mew();
            c.value = 10;
            Box::new(c)
        })
        .add_existing_component(Box::new(c))
        .create();

    assert!(entitiy.has_component::<TestComponent>());
    assert!(entitiy.has_component::<TestComponent1>());

    assert_eq!(
        entitiy
            .get_component::<TestComponent>()
            .unwrap()
            .borrow()
            .value,
        10
    );

    assert_eq!(
        entitiy
            .get_component::<TestComponent1>()
            .unwrap()
            .borrow()
            .value,
        20
    );

    let t = TestComponent::mew();
    let e = EntityBuilder::new()
        .add_component::<TestComponent>()
        .add_existing_component(Box::new(t))
        .create_component(|| Box::new(TestComponent::mew()))
        .create();

    assert_eq!(1, e.components.len());
}

#[test]
fn get_component_test() {
    let mut entity = Entity::new();
    entity.add_component::<TestComponent>().unwrap();
    let c = entity.get_component::<TestComponent>();
    assert!(c.is_some());
    entity.remove_component::<TestComponent>().unwrap();

    let c = entity.get_component::<TestComponent>();
    assert!(c.is_none());
}

#[test]
fn component_update_test() {
    let mut entity = Entity::new();

    entity.add_component::<TestComponent>().unwrap();
    entity.update();

    let c = entity.get_component::<TestComponent>().unwrap();
    let c = c.borrow();
    assert_eq!(c.value, 10)
}

#[test]
fn component_decatification_test() {
    let mut entity = Entity::new();

    entity.add_component::<TestComponent>().unwrap();
    entity.update();

    let c = entity.get_component::<TestComponent>().unwrap();
    assert_eq!(c.borrow().value, 10);

    entity.decatify();
}

#[test]
fn self_refernce_test() {
    let mut world = World::new();

    world.add_entity(
        EntityBuilder::new()
            .add_component::<TestComponent1>()
            .add_component::<TestComponent2>()
            .create(),
    );

    let binding = world
        .get_all_entities_with_component::<TestComponent2>()
        .unwrap();

    let e = binding.first().unwrap();

    let c = e.borrow();
    _ = c.get_component::<TestComponent1>().unwrap();
}
