use std::sync::OnceLock;

use crate as lunar_engine;
use crate::ecs::ComponentReference;

use crate::{
    components::transform::Transform,
    ecs::Component,
    math::{Vec3, Vector},
    structures::{Color, LightBuffer},
};

use lunar_engine_derive::{dependencies, unique};

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
            camera_direction: Vec3::default(),
        }
    }
}

#[derive(Debug)]
///A point light
pub struct PointLight {
    ///Color of the light
    color: Color,
    ///Brightness of the light
    intensity: f32,
    ///Range of the light
    range: f32,
    pub(crate) transform_ref: OnceLock<ComponentReference<Transform>>,
    pub(crate) modified: bool,
}

impl PointLight {
    ///Creates a new point light
    #[must_use]
    pub const fn new(color: Color, intensity: f32, range: f32) -> Self {
        Self {
            color,
            intensity,
            range,
            transform_ref: OnceLock::new(),
            modified: false,
        }
    }

    ///Returns the color of the light
    pub const fn get_color(&self) -> Color {
        self.color
    }
    ///Sets the color of the light
    pub const fn set_color(&mut self, color: Color) {
        self.color = color;
        self.modified = true;
    }

    ///Returns the intensity of the light
    pub const fn get_intensity(&self) -> f32 {
        self.intensity
    }

    ///Sets the intensity of the light
    pub const fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
        self.modified = true;
    }

    ///Returns the range of the light
    pub const fn get_range(&self) -> f32 {
        self.range
    }
    ///Sets the range of the light
    pub const fn set_range(&mut self, range: f32) {
        self.range = range;
        self.modified = true;
    }
}

impl Component for PointLight {
    #[dependencies(Transform)]

    fn mew() -> Self
    where
        Self: Sized,
    {
        Self {
            color: Color::white(),
            intensity: 10.0,
            range: 10.0,
            transform_ref: OnceLock::new(),
            modified: true,
        }
    }

    fn set_self_reference(&mut self, reference: crate::ecs::SelfReferenceGuard) {
        //We can safely unwrap all of this since we can not add this component without adding
        //transform first
        self.transform_ref
            .set(reference.get_component().unwrap())
            .unwrap();
    }
}
