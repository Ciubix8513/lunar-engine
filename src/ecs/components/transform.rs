use renderer_lib::math::vec3::Vec3;

use crate::ecs::component::Component;

#[derive(Debug)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Component for Transform {
    fn mew() -> Self
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
        "Transform"
    }

    fn update(&mut self) {}
    fn awawa(&mut self) {}
    fn decatification(&mut self) {}

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}
