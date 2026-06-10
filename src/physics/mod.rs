//! Physics simulation :3
//!
//!
//!
mod queries;

pub use queries::*;

use std::{any::TypeId, sync::Arc, time::SystemTime};

use log::info;
use nalgebra::{Isometry, Unit, UnitQuaternion, Vector3};
use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use rapier3d::{
    parry::shape::Ball,
    prelude::{
        BroadPhaseBvh, CCDSolver, Collider, ColliderBuilder, ColliderSet, Cuboid,
        DebugRenderBackend, DebugRenderMode, DebugRenderPipeline, ImpulseJointSet,
        IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase, PhysicsPipeline,
        RigidBodyBuilder, RigidBodySet, Shape, SharedShape,
    },
};
use vec_key_value_pair::map::VecMap;

use crate::{
    UUID,
    components::{
        self,
        physics::{self, PhysObject},
        transform::Transform,
    },
    ecs::{
        ComponentReference, WeakEntityRefence, World,
        tracking::{self, EntityEvent, EventQueue},
    },
    math::{Quaternion, Vec3},
};

#[cfg(test)]
mod tests;

struct PhysicsHooks;
struct EventHandler;

impl rapier3d::prelude::PhysicsHooks for PhysicsHooks {}

impl rapier3d::prelude::EventHandler for EventHandler {
    fn handle_collision_event(
        &self,
        _: &RigidBodySet,
        _: &ColliderSet,
        _: rapier3d::prelude::CollisionEvent,
        _: Option<&rapier3d::prelude::ContactPair>,
    ) {
    }

    fn handle_contact_force_event(
        &self,
        _: f32,
        _: &RigidBodySet,
        _: &ColliderSet,
        _: &rapier3d::prelude::ContactPair,
        _: f32,
    ) {
    }
}

///Stores references
struct ComponentStore {
    obj_store: VecMap<UUID, ComponentReference<physics::PhysObject>>,
    col_store: VecMap<UUID, ComponentReference<physics::Collider>>,
}

impl Default for ComponentStore {
    fn default() -> Self {
        Self {
            obj_store: VecMap::new(),
            col_store: VecMap::new(),
        }
    }
}

///Physics handler
pub struct PhysicsState {
    gravity: Vector3<f32>,
    pipeline: PhysicsPipeline,
    parameters: IntegrationParameters,
    phyiscs_sim_end: SystemTime,
    island_manager: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    phys_hooks: PhysicsHooks,
    ev_handler: EventHandler,
    debug_render_pipeline: DebugRenderPipeline,
    comps: ComponentStore,

    world_event_queue: Option<Arc<RwLock<EventQueue>>>,
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self::new()
    }
}

enum UserData {
    Id,
    None,
}

fn to_rapier_shape(shape: components::physics::Shape, scale: Vec3) -> SharedShape {
    match shape {
        crate::components::physics::Shape::Box { dimensions } => SharedShape::new(Cuboid {
            half_extents: (dimensions * scale).into(),
        }),
        crate::components::physics::Shape::Sphere { radius } => SharedShape::new(Ball {
            radius: radius * scale.max(),
        }),
        crate::components::physics::Shape::Capsule => todo!(),
    }
}

fn build_collider(
    c: ComponentReference<components::physics::Collider>,
    local: bool,
    ignore_tranform: bool,
    user_data: UserData,
) -> Collider {
    let c = c.borrow();

    let position = if ignore_tranform {
        Vec3::default()
    } else if local {
        c.transform().borrow().position
    } else {
        c.transform().borrow().position_global()
    };

    let rotation = if ignore_tranform {
        Quaternion::default()
    } else if local {
        c.transform().borrow().rotation
    } else {
        c.transform().borrow().rotation_global()
    };

    let scale = if local {
        c.transform().borrow().scale
    } else {
        c.transform().borrow().scale_global()
    };

    let b = ColliderBuilder::new(to_rapier_shape(c.shape, scale));

    b.mass(1.0)
        .friction(c.material.friction)
        .restitution(c.material.bounciness)
        .position(Isometry::from_parts(
            nalgebra::Translation {
                vector: position.into(),
            },
            UnitQuaternion::from_quaternion(rotation.into()), // rotation.into(),
        ))
        .user_data(match user_data {
            UserData::Id => c.get_id(),
            UserData::None => 0,
        })
        .build()
}

impl PhysicsState {
    ///Creates a new physics handler
    pub fn new() -> Self {
        Self {
            gravity: Vector3::new(0.0, -9.81, 0.0),
            pipeline: PhysicsPipeline::new(),
            parameters: IntegrationParameters::default(),
            phyiscs_sim_end: std::time::SystemTime::now(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            phys_hooks: PhysicsHooks,
            ev_handler: EventHandler,
            debug_render_pipeline: DebugRenderPipeline::new(
                rapier3d::prelude::DebugRenderStyle::default(),
                DebugRenderMode::COLLIDER_SHAPES
                    | DebugRenderMode::RIGID_BODY_AXES
                    | DebugRenderMode::CONTACTS,
            ),
            comps: ComponentStore::default(),
            world_event_queue: None,
        }
    }

    ///Renders physics debug data
    pub fn render(&mut self, backend: &mut impl DebugRenderBackend) {
        self.debug_render_pipeline.render(
            backend,
            &self.bodies,
            &self.colliders,
            &self.impulse_joints,
            &self.multibody_joints,
            &self.narrow_phase,
        );
    }

    ///Sets the gravity of the simulation
    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity.into()
    }

    ///Sets up the simulation with a given world
    pub fn set_up(&mut self, world: &mut World) {
        //A queue for checking changes to the phys componets
        if self.world_event_queue.is_none() {
            let id = world
                .queues
                .write()
                .create_event_queue(tracking::EventFilter::MultipleTypes(vec![
                    TypeId::of::<PhysObject>(),
                    TypeId::of::<physics::Collider>(),
                ]));

            self.world_event_queue = Some(world.queues.read().get_queue(id).unwrap());
        }

        let mut colliders = world.get_all_components::<physics::Collider>();
        let phys_objs = world.get_all_components::<PhysObject>();

        self.setup_objs(phys_objs, colliders);
    }

    fn setup_objs(
        &mut self,
        phys_objs: Vec<ComponentReference<PhysObject>>,
        mut colliders: Vec<ComponentReference<physics::Collider>>,
    ) {
        //Getting a tree of colliders for each phys_obj
        let mut trees = Vec::new();

        //Doing recursion :3
        //this is so silly
        fn traverse_tree(
            t: &ComponentReference<Transform>,
            root: bool,
        ) -> Vec<ComponentReference<physics::Collider>> {
            if t.borrow().enity().has_component::<PhysObject>() && !root {
                return Vec::new();
            }

            let mut o = Vec::new();

            if let Ok(c) = t.borrow().enity().get_component() {
                o.push(c);
            }

            for i in t.borrow().get_children() {
                o.extend(traverse_tree(i, false));
            }

            o
        }

        //Let's assume for now that a phys obj can not have a phys obj child
        for i in &phys_objs {
            let id = i.borrow().get_id();
            let t = i.borrow().transform().clone();

            trees.push((id, traverse_tree(&t, true)));
        }

        //let's isolate singular colliders vs those on phys objs
        for i in trees.iter().flat_map(|i| &i.1).map(|i| i.borrow().get_id()) {
            let mut to_be_removed = None;
            for (n, c) in colliders.iter().enumerate() {
                if c.borrow().get_id() == i {
                    to_be_removed = Some(n);
                }
            }

            if let Some(n) = to_be_removed {
                colliders.remove(n);
            }
        }

        for c in colliders {
            self.colliders
                .insert(build_collider(c, false, false, UserData::Id));
        }
        //now colliders only contain the colliders that do not have a phys obj as an ancestor

        for (i, (_, colliders)) in phys_objs.iter().zip(trees) {
            let o = i.borrow();
            let binding = o.transform();
            let t = binding.borrow();
            let body = RigidBodyBuilder::dynamic()
                .pose(Isometry {
                    translation: t.position_global().into(),
                    rotation: t.rotation.into(),
                })
                .can_sleep(false)
                .gravity_scale(1.0)
                .user_data(o.get_id())
                .build();

            self.comps.obj_store.insert(o.get_id(), i.clone());

            drop(o);
            drop(t);
            let hndl = self.bodies.insert(body);

            for c in colliders {
                let ignore_t = c.borrow().get_id() == c.borrow().get_id();

                let ud = if ignore_t {
                    UserData::None
                } else {
                    UserData::Id
                };
                self.comps.col_store.insert(c.borrow().get_id(), c.clone());

                let c = build_collider(c, true, ignore_t, ud);

                self.colliders.insert_with_parent(c, hndl, &mut self.bodies);
            }
        }
    }

    ///Applies transformations calculated by the physics simulation
    pub fn apply_step(&mut self) {
        for (_hndl, body) in self.bodies.iter() {
            let obj = self
                .comps
                .obj_store
                .get_mut(&body.user_data)
                .unwrap()
                .borrow_mut();
            let binding = obj.transform();
            let mut t = binding.borrow_mut();

            let p: Vec3 = (*body.translation()).into();
            let gp = t.position_global();
            let diff = p - gp;

            t.position += diff;

            let n_q = body.rotation().quaternion();
            let q = n_q.into();

            let r_diff = t.rotation_global() / q;

            t.rotation *= r_diff;

            t.rotation = t.rotation.normalize();
        }
    }

    ///Applies changes of the ECS world onto the physics simulation
    fn apply_world(&mut self) {
        //Sync with the event queue
        if let Some(queue) = &self.world_event_queue {
            let queue = queue.upgradable_read();

            let mut entities = VecMap::<UUID, WeakEntityRefence>::new();

            for e in queue.iter() {
                match e {
                    tracking::Event::Component(component_event) => match component_event {
                        tracking::ComponentEvent::Addition(type_id, any_component_reference) => {
                            todo!()
                        }
                        tracking::ComponentEvent::Removal(type_id) => todo!(),
                    },
                    tracking::Event::Entity(entity_event) => match entity_event {
                        EntityEvent::Addition(id, type_ids, weak) => {
                            entities.insert(*id, weak.clone());
                        }

                        EntityEvent::Removal(id) => {
                            entities.remove(id);
                        }
                    },
                }
            }

            RwLockUpgradableReadGuard::upgrade(queue).clear();

            if !entities.is_empty() {
                let (objs, cols): (Vec<_>, Vec<_>) = entities
                    .values()
                    .map(|e| {
                        let binding = e.upgrade().unwrap();
                        let e = binding.read();
                        (
                            e.get_component::<PhysObject>(),
                            e.get_component::<physics::Collider>(),
                        )
                    })
                    .unzip();

                let objs: Vec<_> = objs.into_iter().flatten().collect();
                let cols: Vec<_> = cols.into_iter().flatten().collect();

                self.setup_objs(objs, cols);
            }
        }
        for (_hndl, body) in self.bodies.iter_mut() {
            let obj = self.comps.obj_store.get(&body.user_data).unwrap().borrow();

            let p = obj.transform().borrow().position_global();
            let r = obj.transform().borrow().rotation_global();
            let s = obj.transform().borrow().scale;

            body.set_position(
                Isometry {
                    rotation: r.into(),
                    translation: p.into(),
                },
                false,
            );

            for c in body.colliders() {
                let shape = self
                    .comps
                    .col_store
                    .get(&body.user_data)
                    .unwrap()
                    .borrow()
                    .shape;
                let collider = self.colliders.get_mut(*c).unwrap();
                collider.set_shape(to_rapier_shape(shape, s));
            }
        }

        for (_hndl, collider) in self.colliders.iter_mut() {
            if collider.user_data == 0 {
                continue;
            }

            if let Some(col) = self.comps.col_store.get(&collider.user_data) {
                let col = col.borrow();
                let p = col.transform().borrow().position_global();
                let r = col.transform().borrow().rotation_global();
                let s = col.transform().borrow().scale;

                collider.set_shape(to_rapier_shape(col.shape, s));

                collider.set_position(Isometry {
                    rotation: r.into(),
                    translation: p.into(),
                });
            }
        }
    }

    ///Step the simulation forward
    pub fn step(&mut self) {
        self.apply_world();
        //Check the world if there were any changes to physics components, and if not just update
        //all the data just in case

        //if yes re do setup and drop the cache

        let dt = self.phyiscs_sim_end.elapsed().unwrap().as_secs_f32();

        self.parameters.dt = 1.0 / 60.0;

        self.pipeline.step(
            &self.gravity,
            &self.parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &self.phys_hooks,
            &self.ev_handler,
        );

        self.phyiscs_sim_end = SystemTime::now();
        self.apply_step();
    }
}
