use crate::math::{mat4x4::Mat4x4, vec3::Vec3};

use crate::ecs::Component;

///Transform  component contains function and data to determine the position of the entity
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
        Self {
            rotation: Vec3::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
            position: Vec3::default(),
        }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

impl Transform {
    ///Returns transformation of the entity
    #[must_use]
    pub fn matrix(&self) -> Mat4x4 {
        Mat4x4::transform_matrix_euler(&self.position, &self.scale, &self.rotation)
    }
}
