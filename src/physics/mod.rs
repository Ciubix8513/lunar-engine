//! Physics simulation :3
//!
//!
//!

use std::time::SystemTime;

use nalgebra::Vector3;
use rapier3d::prelude::{
    BroadPhaseBvh, CCDSolver, ColliderSet, ImpulseJointSet, IntegrationParameters, IslandManager,
    MultibodyJointSet, NarrowPhase, PhysicsPipeline, RigidBodySet,
};

use crate::{components::physics::Collider, ecs::World, math::Vec3};

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
        }
    }

    ///Sets the gravity of the simulation
    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity.into()
    }

    ///Sets up the simulation with a given world
    pub fn set_up(&mut self, world: &mut World) {
        let colliders = world.get_all_components::<Collider>();
    }

    ///Step the simulation forward
    pub fn step(&mut self) {
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
