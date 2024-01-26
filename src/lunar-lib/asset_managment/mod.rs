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

use std::{sync::Arc, thread};

use lock_api::RwLock;
use rand::Rng;
use vec_key_value_pair::VecMap;

use crate::grimoire;

#[cfg(test)]
mod tests;

#[derive(Debug)]
///Error type for asset management
pub enum Error {
    ///Id of an asset is already set
    IdAlreadySet,
    ///Requested asset does not exist or is not registered
    DoesNotExist,
    ///An error ocured during initialization
    ///
    ///The enclosed `Box<dyn std::error::Error>` contains the error that occured
    InitializationError(Box<dyn std::error::Error>),
}

//Potentially use ecs::UUID
///Type the management system uses for a sset IDs
pub type UUID = u128;

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
pub trait Asset: Send + Sync + std::any::Any {
    ///Returns id of the entity
    fn get_id(&self) -> UUID;
    ///Performs initialization of the asset
    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send>>;
    ///Disposes of all all the resources used by the asset
    fn dispose(&mut self);
    ///Sets the id for the asset, may only be called internally, and only once
    fn set_id(&mut self, id: UUID) -> Result<(), Error>;
    ///Returns wether or not the asset is initialized
    fn is_initialized(&self) -> bool;
    //Will not be needed after Rust 1.75.0
    //Cannot be implemented automatically, well... likely can be, but i can't be bothered
    ///Converts trait object to a `std::any::Any` reference
    ///
    ///This function should be implemented as follows
    ///```
    /// fn as_any(&self) -> &dyn std::any::Any {
    ///     self as &dyn std::any::Any
    /// }
    ///```
    fn as_any(&self) -> &dyn std::any::Any;
    ///Converts trait object to a mutable `std::any::Any` reference
    ///
    ///This function should be implemented as follows
    ///```
    ///fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
    ///    self as &mut dyn std::any::Any
    ///}
    ///```
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

///Reference to an asset inside [AssetStore]
pub struct AssetReference<T: 'static> {
    refernce: Arc<RwLock<parking_lot::RawRwLock, Box<dyn Asset + 'static>>>,
    phantom: std::marker::PhantomData<T>,
}

///Type to hide the ugly original type
pub type AssetGuard<'a, T> = lock_api::MappedRwLockReadGuard<'a, parking_lot::RawRwLock, T>;
///Type to hide the ugly original type
pub type AssetGuardMut<'a, T> = lock_api::MappedRwLockWriteGuard<'a, parking_lot::RawRwLock, T>;

impl<T> AssetReference<T> {
    pub fn borrow(&self) -> AssetGuard<'_, T> {
        let read = self.refernce.read();
        lock_api::RwLockReadGuard::<'_, parking_lot::RawRwLock, Box<(dyn Asset + 'static)>>::map(
            read,
            |i| i.as_any().downcast_ref::<T>().unwrap(),
        )
    }

    pub fn borrow_mut(&self) -> AssetGuardMut<'_, T> {
        let write = self.refernce.write();
        lock_api::RwLockWriteGuard::<'_, parking_lot::RawRwLock, Box<(dyn Asset + 'static)>>::map(
            write,
            |i| i.as_any_mut().downcast_mut::<T>().unwrap(),
        )
    }
}

///Asset manager
///
///Manages the initialization of assets, borrowing of assets and disposal of assets
pub struct AssetStore {
    assets: VecMap<
        UUID,
        (
            Arc<RwLock<parking_lot::RawRwLock, Box<dyn Asset>>>,
            std::any::TypeId,
        ),
    >,
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
        self.assets.insert(
            id,
            (
                Arc::new(RwLock::new(Box::new(asset))),
                std::any::TypeId::of::<T>(),
            ),
        );
        id
    }

    ///Initializes all of the assets in the assetstore
    ///
    ///Utilizes threads to initialize assets in parallel
    pub fn intialize_all(&mut self) -> Result<(), Error> {
        let size = self.assets.len();
        let threads = grimoire::NUM_THREADS;
        let binding = self.assets.values().collect::<Vec<_>>();

        let mut chunk_size = size / threads;
        if chunk_size == 0 {
            chunk_size = 1;
        }

        let collect = binding
            .chunks(chunk_size)
            .map(|c| c.iter().map(|i| (*i).clone()).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        let handles = collect
            .into_iter()
            .map(|c| {
                thread::spawn(move || {
                    c.clone()
                        .iter()
                        .map(move |i| i.0.write().initialize())
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();

        for h in handles {
            for r in h.join().unwrap().into_iter() {
                if let Err(r) = r {
                    return Err(Error::InitializationError(r));
                }
            }
        }

        Ok(())
    }

    ///Returns the [AssetReference] to an asset inside the AssetStore by id
    pub fn get_by_id<T: Asset>(&self, id: UUID) -> Result<AssetReference<T>, Error> {
        let this = self.assets.get(&id);
        match this {
            Some(x) => {
                {
                    let mut x = x.0.write();
                    if !x.is_initialized() {
                        let r = x.initialize();
                        if let Err(r) = r {
                            return Err(Error::InitializationError(r));
                        }
                    }
                }
                Ok(AssetReference {
                    refernce: x.0.clone(),
                    phantom: std::marker::PhantomData,
                })
            }
            None => Err(Error::DoesNotExist),
        }
    }

    ///Returns the first asset of type T
    pub fn get_by_type<T: Asset + 'static>(&self) -> Result<AssetReference<T>, Error> {
        let type_id = std::any::TypeId::of::<T>();

        for i in self.assets.values() {
            if i.1 == type_id {
                let mut x = i.0.write();
                if !x.is_initialized() {
                    let r = x.initialize();
                    if let Err(r) = r {
                        return Err(Error::InitializationError(r));
                    }
                }
                return Ok(AssetReference {
                    refernce: i.0.clone(),
                    phantom: std::marker::PhantomData,
                });
            }
        }

        Err(Error::DoesNotExist)
    }

    ///Disposes of the asset with id
    pub fn dispose_by_id(&self, id: UUID) -> Result<(), Error> {
        match self.assets.get(&id) {
            Some(it) => it.0.write().dispose(),
            None => return Err(Error::DoesNotExist),
        };
        Ok(())
    }

    ///Disposes of all assets
    pub fn dispose_all(&self) {
        for a in self.assets.values().map(|v| v.0.clone()) {
            a.write().dispose()
        }
    }
}
