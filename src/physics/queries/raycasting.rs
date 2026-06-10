use std::collections;

use rapier3d::pipeline::QueryFilter;

use crate::{
    ecs::{EntityRefence, World},
    math::Vec3,
    physics::PhysicsState,
};

///A ray used for raycasing
#[derive(Debug)]
pub struct Ray {
    ///Origin of the ray in world space
    pub origin: Vec3,
    ///Direction of the ray
    pub direction: Vec3,
    ///Max distance a ray will travel
    pub max_length: Option<f32>,
}

///The description of a ray hit
pub struct RayHit {
    ///The distance to the hit object
    pub distance: f32,
    ///The entity that was hit
    pub entity: EntityRefence,
}

impl std::fmt::Debug for RayHit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RayHit")
            .field("distance", &self.distance)
            .finish()
    }
}

impl PhysicsState {
    ///Casts a ray
    pub fn ray_cast(&self, world: &World, ray: &Ray) -> Option<RayHit> {
        let filter = QueryFilter::new();
        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.bodies,
            &self.colliders,
            filter,
        );

        let (handle, distance) = query_pipeline.cast_ray(
            &rapier3d::parry::query::Ray {
                origin: ray.origin.into(),
                dir: ray.direction.into(),
            },
            ray.max_length.unwrap_or(f32::MAX),
            true,
        )?;

        let collider = self.colliders.get(handle)?; //.user_data;
        //
        let e_id = collider
            .parent()
            .and_then(|i| self.bodies.get(i))
            .map(|i| i.user_data)
            .unwrap_or(collider.user_data);

        let entity = world.get_entity_by_id(e_id)?;

        Some(RayHit { distance, entity })
    }
}
