use crate::asset_managment::{Asset, AssetStore, UUID};

use super::BindgroupState;

///Trait for implementing materials
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
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
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

impl From<Box<dyn MaterialTrait + 'static + Send + Sync>> for Material {
    fn from(value: Box<dyn MaterialTrait + 'static + Send + Sync>) -> Self {
        Self {
            id: None,
            initialized: false,
            material: value,
        }
    }
}
