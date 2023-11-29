use renderer_lib::math::vec3::Vec3;

use crate::ecs::component::Component;

#[derive(Debug)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Component for Transform {
    fn new() -> Self
    where
        Self: Sized,
    {
        Transform {
            rotation: Vec3::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
            position: Vec3::default(),
        }
    }

    fn name(&self) -> &'static str {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }

    fn awake(&mut self) {
        todo!()
    }

    fn death(&mut self) {
        todo!()
    }
}
