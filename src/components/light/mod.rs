use crate::{
    ecs::Component,
    math::{Vec3, Vector},
    structures::{Color, LightBuffer},
};

use lunar_engine_derive::{as_any, unique};

///The directional light component, describes the behaviour of the main directional light of a
///scene, i.e. the sun
#[derive(Debug)]
pub struct DirectionalLight {
    ///Direction from which the light is shining
    pub direction: Vec3,
    ///Color of the light
    pub color: Color,
    ///Intensity of the light
    pub intensity: f32,

    ///Color of the ambient light
    pub ambient_color: Color,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Color::white(),
            intensity: 1.0,
            ambient_color: Color::new(0.1, 0.1, 0.1, 1.0),
        }
    }
}

impl Component for DirectionalLight {
    #[as_any]
    #[unique]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn awawa(&mut self) {}
}

impl DirectionalLight {
    pub(crate) fn get_light(&self) -> LightBuffer {
        LightBuffer {
            direction: self.direction.normalize(),
            color: self.color,
            intensity: self.intensity,
            ambient_color: self.ambient_color,
        }
    }
}
