use std::sync::OnceLock;

use lunar_engine::{
    asset_managment::AssetStore,
    assets,
    components::{
        self, camera::MainCamera, light::DirectionalLight, physics::Collider, transform::Transform,
    },
    delta_time,
    ecs::{Component, ComponentReference, Entity, EntityBuilder, World},
    input::{self, CursorLock, CursorVisibily, KeyState},
    math::{Quaternion, Vec3, Vector},
    physics::PhysicsState,
    rendering::{
        extensions::{self, Base},
        render,
    },
};
use winit::keyboard::KeyCode;

struct CameraControls {
    transform: OnceLock<ComponentReference<Transform>>,
}

impl Component for CameraControls {
    fn mew() -> Self
    where
        Self: Sized,
    {
        input::set_cursor_grab_mode(CursorLock::Locked);
        input::set_cursor_visible(CursorVisibily::Hidden);
        Self {
            transform: OnceLock::new(),
        }
    }

    fn update(&mut self) {
        //Cursor stuff
        if input::key(KeyCode::Escape) == KeyState::Down {
            input::set_cursor_grab_mode(CursorLock::Free);
            input::set_cursor_visible(CursorVisibily::Visible);
        }

        if input::mouse_btn(winit::event::MouseButton::Left) == KeyState::Down {
            input::set_cursor_grab_mode(CursorLock::Locked);
            input::set_cursor_visible(CursorVisibily::Hidden);
        }

        let delta_time = delta_time();

        //Rotation
        let sensetivity = 800.0;
        let delta = input::cursor_delta() * delta_time * sensetivity;
        let mut trans = self.transform.get().unwrap().borrow_mut();

        //Using a parent for y axis rotation...
        //
        //kinda scuffed but eh, should work
        let parent = trans.get_parent().clone().unwrap();
        let mut p = parent.borrow_mut();

        // trans.rotate((delta.y * 0.1, delta.x * -0.1, 0.0).into());
        trans.rotate((delta.y * 0.1, 0.0, 0).into());
        p.rotate((0, delta.x * -0.1, 0).into());

        drop(p);

        //Movement
        let mut speed = 400.0;
        if input::key(KeyCode::ShiftLeft) == KeyState::Pressed {
            speed *= 2.0;
        }

        let mut movement_vec = Vec3::default();
        if input::key(KeyCode::KeyW) == KeyState::Pressed {
            movement_vec.z += 1.0;
        }
        if input::key(KeyCode::KeyS) == KeyState::Pressed {
            movement_vec.z -= 1.0;
        }
        if input::key(KeyCode::KeyA) == KeyState::Pressed {
            movement_vec.x += 1.0;
        }
        if input::key(KeyCode::KeyD) == KeyState::Pressed {
            movement_vec.x -= 1.0;
        }
        if input::key(KeyCode::KeyE) == KeyState::Pressed {
            movement_vec.y += 1.0;
        }
        if input::key(KeyCode::KeyQ) == KeyState::Pressed {
            movement_vec.y -= 1.0;
        }

        if movement_vec.square_length() == 0.0 {
            return;
        }

        movement_vec *= 0.01 * speed * delta_time;

        let mat = trans.rotation_global().matrix();
        movement_vec = mat.transform3(movement_vec);

        parent.borrow_mut().position += movement_vec;
    }

    fn set_self_reference(&mut self, reference: lunar_engine::ecs::SelfReferenceGuard) {
        self.transform
            .set(reference.get_component::<Transform>().unwrap())
            .unwrap();
    }
}

#[derive(Default)]
struct State {
    world: World,
    assets: AssetStore,
    extension: Base,
    dbg_ext: extensions::Debug,
    phys_world: PhysicsState,
}
fn run(state: &mut State) {
    state.world.update();

    state.phys_world.render(&mut state.dbg_ext);

    render(
        &mut state.world,
        &mut state.assets,
        &mut [&mut state.extension, &mut state.dbg_ext],
    );
}
fn end(_: &mut State) {}

fn init(state: &mut State) {
    let assets = &mut state.assets;
    let world = &mut state.world;

    let b = assets.register(assets::Mesh::new_box(0.5.into()));
    let m = assets.register(assets::materials::Lit::new(None, None, None, 0.5));

    let e = EntityBuilder::new()
        .add_existing_component(Transform::new(
            0.0.into(),
            Quaternion::default(),
            0.0.into(),
        ))
        .create()
        .unwrap();
    let p = e.get_component().unwrap();

    world.add_entity(e).unwrap();
    world
        .add_entity(
            EntityBuilder::new()
                .create_component(|| {
                    let mut t = Transform::default();
                    t.set_parent(p);
                    t
                })
                .add_component::<MainCamera>()
                .add_component::<CameraControls>()
                .create()
                .unwrap(),
        )
        .unwrap();

    world
        .add_entity(
            EntityBuilder::new()
                .add_component::<DirectionalLight>()
                .create()
                .unwrap(),
        )
        .unwrap();

    world
        .add_entity(
            EntityBuilder::new()
                .add_component::<Transform>()
                .create_component(|| components::mesh::Mesh::new(b, m))
                .create_component(|| {
                    Collider::new(components::physics::Shape::Box {
                        dimensions: 0.5.into(),
                    })
                })
                .create()
                .unwrap(),
        )
        .unwrap();

    state.phys_world.set_up(world);
}

fn main() {
    let state = lunar_engine::State::new(State::default());

    state.run(init, run, end);
}
