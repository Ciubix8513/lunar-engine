// Jesus Christ what am i getting myself into
//! The system for managing assets such as textures, meshes and materials
//!
//! # System description
//!
//! Each asset has a unique ID, by which it may be queried from the asset store
//!
//! Each asset has an initialization function and a disposal function
//!
//! All assets are disposed when the asset store goes out of scope(is disposed off)
//!
//! Most assets are borrowed immutably with only a few exceptions
//!
//! Asset initialization may be performed in parallel
//! Assets are only initialized when first needed (or perhaps on "scene load"?)
// Oh god, is this just the entity system but with assets!?!?

use std::{cell::RefCell, rc::Rc};

use rand::Rng;
use vec_key_value_pair::VecMap;

#[derive(Debug)]
pub enum Error {
    IdAlreadySet,
}

//Potentially use ecs::UUID
///Type the management system uses for a sset IDs
type UUID = u128;

//Send and sync for parallel initialization
///Trait all assets must implement
///
///# Implementation guidelines
///All the heavy initialization work MUST be performed in the `initialize` function
///Before that only minimal work should be performed
///
///For example, if loading and parsing a file, the `initialize` function should perform all the
///file system and parsing operations
///
///The initialize function and the dispose function MUST be able to be called multiple times in
///sequence. i.e.
///
///```
/// let asset = TestAsset::new("filepath");
/// //Read and parse the file
/// asset.initialize();
/// //dispose of all the read data
/// asset.dispose();
/// //Read and parse the file again
/// asset.initialize();
///```
///
///ID must not be set before the asset is registered
pub trait Asset: Send + Sync {
    ///Returns id of the entity
    fn get_id(&self) -> UUID;
    ///Performs initialization of the asset
    fn initialize(&mut self);
    ///Disposes of all all the resources used by the asset
    fn dispose(&mut self);
    ///Sets the id for the asset, may only be called internally, and only once
    fn set_id(&mut self, id: UUID) -> Result<(), Error>;
}

///Asset manager
///
///Manages the initialization of assets, borrowing of assets and disposal of assets
pub struct AssetStore {
    assets: VecMap<UUID, Rc<RefCell<Box<dyn Asset>>>>,
}

impl Default for AssetStore {
    fn default() -> Self {
        Self {
            assets: VecMap::new(),
        }
    }
}

impl AssetStore {
    ///Creates a new asset store
    pub fn new() -> Self {
        Self::default()
    }

    ///Registers a new asset in the store
    pub fn register<T>(&mut self, asset: T) -> UUID
    where
        T: Asset + 'static,
    {
        let id = rand::thread_rng().gen();
        let mut asset = asset;
        asset.set_id(id).unwrap();
        self.assets
            .insert(id, Rc::new(RefCell::new(Box::new(asset))));
        id
    }
}
