use lunar_engine_derive::{alias, dependencies, unique};

use crate as lunar_engine;

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

    fn set_self_reference(&mut self, reference: SelfReferenceGuard) {
        self.weak = Some(reference);
    }
}

#[derive(Debug)]
struct TestComponent3;

impl Component for TestComponent3 {
    #[dependencies(TestComponent)]
    fn mew() -> Self
    where
        Self: Sized,
    {
        Self
    }
}

#[derive(Debug)]
struct TestComponent4 {
    test_comp: Option<ComponentReference<TestComponent>>,
}

impl Component for TestComponent4 {
    #[dependencies(TestComponent)]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self { test_comp: None }
    }

    fn set_self_reference(&mut self, reference: SelfReferenceGuard) {
        self.test_comp = Some(reference.get_component().unwrap())
    }
}

#[test]
fn component_dependency_test() {
    let mut e = Entity::new();

    let res = e.add_component::<TestComponent3>();
    assert_eq!(res, Err(Error::MissingDependency("TestComponent")));

    let res = e.add_component::<TestComponent>();
    assert_eq!(res, Ok(()));

    let res = e.add_component::<TestComponent3>();
    assert_eq!(res, Ok(()));

    let res = EntityBuilder::new()
        .add_component::<TestComponent3>()
        .create();
    if let Err(res) = res {
        assert_eq!(res, Error::MissingDependency("TestComponent"));
    } else {
        assert!(false, "Failed to fail");
    }
    let res = EntityBuilder::new()
        .add_component::<TestComponent>()
        .add_component::<TestComponent3>()
        .create();

    assert!(res.is_ok());
}

#[test]
fn add_enitity_test() {
    let e = Entity::new();
    let mut w = World::new();

    w.add_entity(e).unwrap();

    assert_eq!(w.get_entity_count(), 1);
}

#[test]
fn remove_enitity_test() {
    let e = Entity::new();
    let id = e.get_id();
    let mut w = World::new();

    assert!(w.remove_entity_by_id(id).is_err());

    w.add_entity(e).unwrap();

    w.remove_entity_by_id(id).unwrap();

    assert_eq!(w.get_entity_count(), 0);
}

#[test]
fn get_entity_test() {
    let mut e = Entity::new();
    e.add_component::<TestComponent>().unwrap();
    let id = e.get_id();
    let mut w = World::new();
    w.add_entity(e).unwrap();

    let e = w.get_entity_by_id(id).unwrap();

    let e1 = e.read();
    let c = e1.get_component::<TestComponent>().unwrap();
    drop(e1);
    _ = c.borrow();

    w.remove_entity_by_id(id).unwrap();
}

#[test]
fn get_all_componenents_test() {
    let mut w = World::new();
    let o = w.get_all_components::<TestComponent>();
    assert!(o.is_empty());

    for _ in 0..200 {
        let mut e = Entity::new();
        e.add_component::<TestComponent>().unwrap();
        w.add_entity(e).unwrap();
    }

    //To test speed
    for _ in 0..10000 {
        let o = w.get_all_components::<TestComponent>();
        assert_eq!(o.len(), 200);

        let o = w.get_all_entities_with_component::<TestComponent>();
        assert_eq!(o.len(), 200);
        // w.modified.borrow_mut().entity_changed();
    }

    let mut e = Entity::new();
    e.add_component::<TestComponent>().unwrap();
    w.add_entity(e).unwrap();

    //Test cache invalidation for components
    let o = w.get_all_components::<TestComponent>();
    assert_eq!(o.len(), 201);

    //Test cache invalidation for components
    let o = w.get_all_entities_with_component::<TestComponent>();
    assert_eq!(o.len(), 201);

    let mut e = o.first().unwrap().write();
    e.remove_component::<TestComponent>().unwrap();
    drop(e);

    let o = w.get_all_components::<TestComponent>();
    assert_eq!(o.len(), 200);

    let o = w.get_all_entities_with_component::<TestComponent>();
    assert_eq!(o.len(), 200);
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
            c
        })
        .add_existing_component(c)
        .create()
        .unwrap();

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
        .add_existing_component(t)
        .create_component(TestComponent::mew)
        .create()
        .unwrap();

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
    assert_eq!(c.value, 10);
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

    world
        .add_entity(
            EntityBuilder::new()
                .add_component::<TestComponent1>()
                .add_component::<TestComponent2>()
                .create()
                .unwrap(),
        )
        .unwrap();

    let binding = world.get_all_entities_with_component::<TestComponent2>();

    let e = binding.first().unwrap();

    let c = e.read();
    _ = c.get_component::<TestComponent1>().unwrap();
}

#[derive(Debug)]
#[alias(TestComponent2)]
struct Alias;

#[test]
fn alias_test() {
    let mut world = World::new();

    world
        .add_entity(
            EntityBuilder::new()
                .create_component(|| TestComponent1 { value: 0 })
                .add_component::<Alias>()
                .create()
                .unwrap(),
        )
        .unwrap();

    let binding = world.get_all_components::<Alias>();
    assert_eq!(binding.len(), 1);
}

struct UniqueComponent;

impl Component for UniqueComponent {
    #[unique]

    fn mew() -> Self {
        Self
    }
}

#[test]
fn test_unique_component() {
    let mut world = World::new();

    //Add a unique component
    let res = world.add_entity(
        EntityBuilder::new()
            .add_component::<UniqueComponent>()
            .create()
            .unwrap(),
    );
    //Succeed
    assert!(res.is_ok());

    let res = world.add_entity(
        EntityBuilder::new()
            .add_component::<UniqueComponent>()
            .create()
            .unwrap(),
    );
    //Try again and fail
    assert_eq!(res.err(), Some(Error::UniqueComponentExists));

    //Clear the world
    world.destroy_all();

    let e = world.add_entity(Entity::new()).unwrap();
    let e1 = world.add_entity(Entity::new()).unwrap();

    let res = e
        .upgrade()
        .unwrap()
        .write()
        .add_component::<UniqueComponent>();

    assert_eq!(res, Ok(()));

    let res = e1
        .upgrade()
        .unwrap()
        .write()
        .add_component::<UniqueComponent>();

    assert_eq!(res, Err(Error::UniqueComponentExists));

    let id = e.upgrade().unwrap().read().get_id();

    world.remove_entity_by_id(id).unwrap();

    let res = e1
        .upgrade()
        .unwrap()
        .write()
        .add_component::<UniqueComponent>();

    assert_eq!(res, Ok(()));
}

#[test]
fn get_unique_component() {
    let mut world = World::new();

    world
        .add_entity(
            EntityBuilder::new()
                .add_component::<TestComponent>()
                .create()
                .unwrap(),
        )
        .unwrap();
    world
        .add_entity(
            EntityBuilder::new()
                .add_component::<TestComponent>()
                .create()
                .unwrap(),
        )
        .unwrap();

    let c = world.get_unique_component::<TestComponent>();
    assert!(c.is_none());

    let c = world.get_unique_component::<UniqueComponent>();
    assert!(c.is_none());

    world
        .add_entity(
            EntityBuilder::new()
                .add_component::<UniqueComponent>()
                .create()
                .unwrap(),
        )
        .unwrap();

    let c = world.get_unique_component::<UniqueComponent>();
    assert!(c.is_some());
}

#[test]
fn component_addition() {
    let mut world = World::new();

    let e = world.add_entity(Entity::new()).unwrap();

    //Add dependency
    e.upgrade()
        .unwrap()
        .write()
        .add_component::<TestComponent>()
        .unwrap();

    //Add the component itself
    e.upgrade()
        .unwrap()
        .write()
        .add_component::<TestComponent4>()
        .unwrap();
}
