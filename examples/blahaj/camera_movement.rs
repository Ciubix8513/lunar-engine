use lunar_engine::{
    components::transform::Transform,
    ecs::{Component, ComponentReference, SelfReferenceGuard},
};

#[derive(Debug)]
pub struct FreeCam {
    speed: f32,
    sensetivity: f32,
    transorm_reference: Option<ComponentReference<Transform>>,
}

impl FreeCam {
    pub fn new(speed: f32, sensetivity: f32) -> Self {
        Self {
            speed,
            sensetivity,
            transorm_reference: None,
        }
    }
}

impl Component for FreeCam {
    fn mew() -> Self
    where
        Self: Sized,
    {
        FreeCam {
            speed: 1.0,
            sensetivity: 1.0,
            transorm_reference: None,
        }
    }

    fn set_self_reference(&mut self, reference: SelfReferenceGuard) {
        self.transorm_reference = Some(reference.get_component().unwrap())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn update(&mut self) {}
}
