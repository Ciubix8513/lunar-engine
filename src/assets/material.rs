use lunar_engine_derive::as_any;

use crate::asset_managment::{Asset, AssetStore, UUID};

use super::BindgroupState;

///Trait for implementing materials
#[allow(clippy::module_name_repetitions)]
pub trait MaterialTrait {
    ///Render function of the material
    fn render(&self, render_pass: &mut wgpu::RenderPass);
    ///Initialization of the material
    fn intialize(&mut self);
    ///Disposal of the material
    fn dispose(&mut self);
    ///Creation of bindgroups and populating them with data
    fn set_bindgroups(&mut self, asset_store: &AssetStore);
    ///State of the bindgroups of the material
    fn bindgroup_sate(&self) -> BindgroupState;
}

///Stores material data, wrapper around the material trait object
pub struct Material {
    id: Option<UUID>,
    initialized: bool,
    material: Box<dyn MaterialTrait + Sync + Send>,
}

impl Asset for Material {
    #[as_any]

    fn get_id(&self) -> UUID {
        self.id.unwrap()
    }

    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send>> {
        self.material.intialize();
        self.initialized = true;
        Ok(())
    }

    fn dispose(&mut self) {
        self.material.dispose();
        self.initialized = false;
    }

    fn set_id(&mut self, id: UUID) -> Result<(), crate::asset_managment::Error> {
        if self.id.is_some() {
            Err(crate::asset_managment::Error::IdAlreadySet)
        } else {
            self.id = Some(id);
            Ok(())
        }
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Material {
    #[must_use]
    ///Get the bindgroup state of the material
    pub fn get_bindgroup_state(&self) -> BindgroupState {
        self.material.bindgroup_sate()
    }

    ///Initialize bindgroups of the material
    pub fn initialize_bindgroups(&mut self, asset_store: &AssetStore) {
        self.material.set_bindgroups(asset_store);
    }

    ///Call the render function of the material
    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        self.material.render(render_pass);
    }
}

impl<T> From<T> for Material
where
    T: MaterialTrait + Send + Sync + 'static,
{
    fn from(value: T) -> Self {
        Self {
            id: None,
            initialized: false,
            material: Box::new(value),
        }
    }
}
