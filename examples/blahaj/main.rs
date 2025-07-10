use std::{cell::OnceCell, path::Path};

use log::{debug, info};
use lunar_engine::{
    State,
    asset_managment::AssetStore,
    assets::{self, materials::Unlit},
    components::{
        camera::{MainCamera, ProjectionType},
        mesh::Mesh,
        transform::Transform,
    },
    ecs::{Component, ComponentReference, EntityBuilder, World},
    input,
    math::{Quaternion, Vec3},
    rendering::{self, extensions::Base},
    structures::Color,
};
use lunar_engine_derive::{dependencies, marker_component};
use winit::keyboard::KeyCode;

#[derive(Default)]
struct MyState {
    frame: u32,
    world: World,
    assset_store: AssetStore,
    extension: Base,
    blahaj_mesh: u128,
    blahaj_mat: u128,
}

#[marker_component]
struct Blahaj;

struct Spiny {
    pub speed: f32,
    transform: OnceCell<ComponentReference<Transform>>,
}

impl Component for Spiny {
    #[dependencies(Transform)]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self {
            speed: 100.0,
            transform: OnceCell::new(),
        }
    }

    fn set_self_reference(&mut self, reference: lunar_engine::ecs::SelfReferenceGuard) {
        self.transform
            .set(reference.get_component().unwrap())
            .unwrap();
    }

    fn update(&mut self) {
        let mut t = self.transform.get().unwrap().borrow_mut();
        let r = t.rotation.euler();

        println!("rot = {r}");

        let r1 = (r.x, r.y + self.speed * lunar_engine::delta_time(), r.z).into();

        println!("rot1 = {r1}");

        t.rotation = Quaternion::from_euler(r1);
    }
}

fn init(state: &mut MyState) {
    //Set Vsync
    lunar_engine::set_vsync(lunar_engine::Vsync::Vsync);

    log::info!("Initializing scene");

    state.extension = Base::new_with_color(
        0,
        true,
        Color {
            r: 0.96,
            g: 0.65,
            b: 0.72,
            a: 1.0,
        },
    );

    state.frame = 0;
    let mesh = state
        .assset_store
        .register(assets::Mesh::new_from_obj(Path::new("assets/blahaj.obj")).unwrap());
    let texture = state
        .assset_store
        .register(assets::Texture::new_png(Path::new("assets/blahaj.png")));
    let material = state.assset_store.register(Unlit::new(Some(texture), None));

    state.blahaj_mat = material;
    state.blahaj_mesh = mesh;
    let _e = state.world.add_entity(
        EntityBuilder::new()
            .create_component(|| Transform {
                position: Vec3::new(0, 2, -10),
                rotation: Quaternion::from_euler(Vec3::new(15, 0, 0)),
                ..Default::default()
            })
            .create_component(|| {
                let mut c = MainCamera::mew();
                //60 degree FOV
                c.projection_type = ProjectionType::Perspective {
                    fov: std::f32::consts::FRAC_PI_3,
                };
                c.near = 0.1;
                c.far = 100.0;
                c
            })
            .create()
            .unwrap(),
    );

    state
        .world
        .add_entity(
            EntityBuilder::new()
                .create_component(Transform::default)
                .create_component(|| Mesh::new(state.blahaj_mesh, state.blahaj_mat))
                .add_component::<Blahaj>()
                .add_component::<Spiny>()
                .create()
                .unwrap(),
        )
        .unwrap();

    info!("Initialized!");
    info!("World contains {} entities", state.world.get_entity_count());

    if state.world.get_all_components::<MainCamera>().is_some() {
        info!("World contains a main camera");
    }
}

fn run(state: &mut MyState) {
    if input::KeyState::Down == input::key(KeyCode::KeyB) {
        state
            .world
            .add_entity(
                EntityBuilder::new()
                    .create_component(|| Transform {
                        // scale: Vec3::random(0.3, 3.0),
                        rotation: Quaternion::from_euler(Vec3::random(0, 360)),
                        position: Vec3::random(-5, 5),
                        ..Default::default()
                    })
                    .create_component(|| Mesh::new(state.blahaj_mesh, state.blahaj_mat))
                    .add_component::<Blahaj>()
                    .create()
                    .unwrap(),
            )
            .unwrap();
    }

    if input::KeyState::Down == input::key(KeyCode::KeyC) {
        let e = state
            .world
            .get_all_entities_with_component::<Blahaj>()
            .unwrap_or_default();
        for b in e {
            let id = b.borrow().get_id();
            state.world.remove_entity_by_id(id).unwrap();
        }
    }

    state.world.update();
    debug!("Called render!");
    rendering::render(
        &state.world,
        &mut state.assset_store,
        &mut [&mut state.extension],
    );
    state.frame += 1;
}

fn close(state: &mut MyState) {
    info!("Ran for {} frames", state.frame);
}

fn main() {
    let state = State::<MyState>::default();
    state.run(init, run, close);
}
