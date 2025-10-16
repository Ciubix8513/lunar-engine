//! Physics simulation :3
//!
//!
//!

use std::time::SystemTime;

use nalgebra::{Isometry, Unit, UnitQuaternion, Vector3};
use rapier3d::prelude::{
    BroadPhaseBvh, CCDSolver, ColliderBuilder, ColliderSet, DebugRenderBackend,
    DebugRenderPipeline, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
    NarrowPhase, PhysicsPipeline, RigidBodySet,
};

use crate::{
    components::{
        physics::{Collider, PhysObject},
        transform::Transform,
    },
    ecs::{ComponentReference, World},
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
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self::new()
    }
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
            debug_render_pipeline: DebugRenderPipeline::render_all(
                rapier3d::prelude::DebugRenderStyle::default(),
            ),
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
        let mut colliders = world.get_all_components::<Collider>();
        let phys_objs = world.get_all_components::<PhysObject>();

        //Getting a tree of colliders for each phys_obj
        let mut trees = Vec::new();

        //Doing recursion :3
        //this is so silly
        fn traverse_tree(
            t: &ComponentReference<Transform>,
            root: bool,
        ) -> Vec<ComponentReference<Collider>> {
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
            let c = c.borrow();

            let b = match c.shape {
                crate::components::physics::Shape::Box { dimensions } => {
                    ColliderBuilder::cuboid(dimensions.x, dimensions.y, dimensions.z)
                }
                crate::components::physics::Shape::Sphere { radius } => {
                    ColliderBuilder::ball(radius)
                }
                crate::components::physics::Shape::Capsule => todo!(),
            };

            let position = c.transform().borrow().position_global();
            let rotation = c.transform().borrow().rotation_global();

            self.colliders.insert(
                b.mass(0.0)
                    .friction(c.material.friction)
                    .restitution(c.material.bounciness)
                    .position(Isometry::from_parts(
                        nalgebra::Translation {
                            vector: position.into(),
                        },
                        UnitQuaternion::from_quaternion(rotation.into()), // rotation.into(),
                    ))
                    .user_data(c.get_id()),
            );
        }

        //now colliders only contain the colliders that do not have a phys obj as an ancestor
    }

    ///Step the simulation forward
    pub fn step(&mut self) {
        //Check the world if there were any changes to physics components, and if not just update
        //all the data just in case

        //if yes re do setup and drop the cache

        let dt = self.phyiscs_sim_end.elapsed().unwrap().as_secs_f32();

        self.parameters.dt = dt;

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

        self.phyiscs_sim_end = SystemTime::now()
    }
}
