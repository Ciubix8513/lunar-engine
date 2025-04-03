#![allow(clippy::ref_as_ptr, clippy::ptr_as_ptr)]
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

use std::{
    any::Any,
    sync::{Arc, Weak},
};

#[cfg(not(target_arch = "wasm32"))]
use std::thread;

use rand::Rng;
use vec_key_value_pair::map::VecMap;

#[cfg(not(target_arch = "wasm32"))]
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
///# use lunar_engine::asset_managment::Asset;
///# use std::any::Any;
///# struct TestAsset;
///# impl Asset for TestAsset {
///# fn get_id(&self) -> u128 { todo!() }
///# fn initialize(&mut self) -> Result<(), Box<(dyn std::error::Error + Send + 'static)>> { Ok(()) }
///# fn dispose(&mut self) { }
///# fn set_id(&mut self, _: u128) -> Result<(), lunar_engine::asset_managment::Error> { todo!() }
///# fn is_initialized(&self) -> bool { todo!() }
///# }
///# impl TestAsset { fn new(_ : &str) -> Self { Self }}
/// let mut asset = TestAsset::new("filepath");
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
    ///Returns id of the asset
    fn get_id(&self) -> UUID;
    ///Performs initialization of the asset
    ///
    ///# Errors
    ///May return an error if the initialization of an asset fails
    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send>>;
    ///Disposes of all all the resources used by the asset
    fn dispose(&mut self);
    ///Sets the id for the asset, may only be called internally, and only once
    ///
    ///# Errors
    ///Returns an error if the id was already set
    fn set_id(&mut self, id: UUID) -> Result<(), Error>;
    ///Returns whether or not the asset is initialized
    fn is_initialized(&self) -> bool;
}

///Reference to an asset inside [`AssetStore`]
pub struct AssetReference<T: 'static> {
    refernce: Weak<RwLock<Box<dyn Asset + 'static>>>,
    phantom: std::marker::PhantomData<T>,
}

///Type to hide the ugly original type
pub type AssetGuard<'a, T> = lock_api::MappedRwLockReadGuard<'a, parking_lot::RawRwLock, T>;
///Type to hide the ugly original type
pub type AssetGuardMut<'a, T> = lock_api::MappedRwLockWriteGuard<'a, parking_lot::RawRwLock, T>;

impl<T> AssetReference<T> {
    ///Borrows the asset immutably
    #[inline(always)]
    pub fn borrow(&self) -> AssetGuard<'_, T> {
        // let read = self.refernce.read();
        lock_api::RwLockReadGuard::<'_, parking_lot::RawRwLock, Box<(dyn Asset + 'static)>>::map(
            unsafe { self.refernce.as_ptr().as_ref().unwrap().read() },
            |i| unsafe { &*(i as *const dyn Any as *const T) },
        )
    }

    ///Borrows the asset mutably
    #[allow(clippy::ref_as_ptr, clippy::ptr_as_ptr)]
    #[inline(always)]
    pub fn borrow_mut(&self) -> AssetGuardMut<'_, T> {
        lock_api::RwLockWriteGuard::<'_, parking_lot::RawRwLock, Box<(dyn Asset + 'static)>>::map(
            unsafe { self.refernce.as_ptr().as_ref().unwrap().write() },
            |i| unsafe { &mut *(i as *mut dyn Any as *mut T) },
        )
    }
}

type RwLock<T> = lock_api::RwLock<parking_lot::RawRwLock, T>;

///Asset manager
///
///Manages the initialization of assets, borrowing of assets and disposal of assets
#[allow(clippy::type_complexity)]
pub struct AssetStore {
    assets: VecMap<UUID, (Arc<RwLock<Box<dyn Asset>>>, std::any::TypeId)>,
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    ///Registers a new asset in the store
    ///
    ///# Panics
    ///Panics if the id of the asset was previously set
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
    ///
    ///# Errors
    ///Returns an error if one of the assets fails to initialize
    pub fn intialize_all(&self) -> Result<(), Error> {
        let binding = self.assets.values().collect::<Vec<_>>();

        #[cfg(not(target_arch = "wasm32"))]
        let mut chunk_size = self.assets.len() / grimoire::NUM_THREADS;
        #[cfg(not(target_arch = "wasm32"))]
        if chunk_size == 0 {
            chunk_size = 1;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let handles = binding
                .chunks(chunk_size)
                .map(|c| c.iter().map(|i| (*i).clone()).collect::<Vec<_>>())
                .collect::<Vec<_>>()
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
                for r in unsafe { h.join().unwrap_unchecked() } {
                    if let Err(r) = r {
                        return Err(Error::InitializationError(r));
                    }
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            for r in binding.iter().map(|c| c.0.write().initialize()) {
                if let Err(r) = r {
                    return Err(Error::InitializationError(r));
                }
            }
        }

        Ok(())
    }

    ///Returns the [`AssetReference`] to an asset inside the `AssetStore` by id
    ///
    ///# Errors
    ///Returns an error if the object with the given id doesn't exist
    pub fn get_by_id<T: Asset>(&self, id: UUID) -> Result<AssetReference<T>, Error> {
        let this = self.assets.get(&id);
        match this {
            Some(x) => {
                {
                    let mut x = x.0.write();
                    if !x.is_initialized() {
                        let r = x.initialize();
                        drop(x);
                        if let Err(r) = r {
                            return Err(Error::InitializationError(r));
                        }
                    }
                }
                Ok(AssetReference {
                    refernce: Arc::downgrade(&x.0),
                    phantom: std::marker::PhantomData,
                })
            }
            None => Err(Error::DoesNotExist),
        }
    }

    #[allow(clippy::ptr_as_ptr, clippy::ref_as_ptr)]
    ///Borrows an asset by its id, same as `get_by_id`, but with the `borrow` call is already made
    ///
    ///# Errors
    ///Returns an error if the object with the given id doesn't exist
    #[inline(always)]
    pub fn borrow_by_id<T: Asset>(&self, id: UUID) -> Result<AssetGuard<T>, Error> {
        let this = self.assets.get(&id);
        match this {
            Some(x) => {
                {
                    let mut x = x.0.write();
                    if !x.is_initialized() {
                        let r = x.initialize();
                        drop(x);
                        if let Err(r) = r {
                            return Err(Error::InitializationError(r));
                        }
                    }
                }
                Ok({
                    lock_api::RwLockReadGuard::<
                        '_,
                        parking_lot::RawRwLock,
                        Box<(dyn Asset + 'static)>,
                    >::map(x.0.read(), |i| unsafe {
                        &*(i as *const dyn Any as *const T)
                    })
                })
            }
            None => Err(Error::DoesNotExist),
        }
    }

    ///Borrows an asset by its id, same as `get_by_id`, but with the `borrow_mut` call is already made
    ///
    ///# Errors
    ///Returns an error if the object with the given id doesn't exist
    #[inline(always)]
    pub fn borrow_by_id_mut<T: Asset>(&self, id: UUID) -> Result<AssetGuardMut<T>, Error> {
        let this = self.assets.get(&id);
        match this {
            Some(x) => {
                {
                    let mut x = x.0.write();
                    if !x.is_initialized() {
                        let r = x.initialize();
                        drop(x);
                        if let Err(r) = r {
                            return Err(Error::InitializationError(r));
                        }
                    }
                }
                Ok({
                    lock_api::RwLockWriteGuard::<
                        '_,
                        parking_lot::RawRwLock,
                        Box<(dyn Asset + 'static)>,
                    >::map(x.0.write(), |i| unsafe {
                        &mut *(i as *mut dyn Any as *mut T)
                    })
                })
            }
            None => Err(Error::DoesNotExist),
        }
    }

    ///Returns the first asset of type T
    ///
    ///# Errors
    ///Returns an error if the object of the given type doesn't exist
    pub fn get_by_type<T: Asset + 'static>(&self) -> Result<AssetReference<T>, Error> {
        let type_id = std::any::TypeId::of::<T>();

        for i in self.assets.values() {
            if i.1 == type_id {
                let mut x = i.0.write();
                if !x.is_initialized() {
                    let r = x.initialize();
                    drop(x);
                    if let Err(r) = r {
                        return Err(Error::InitializationError(r));
                    }
                }
                return Ok(AssetReference {
                    refernce: Arc::downgrade(&i.0),
                    phantom: std::marker::PhantomData,
                });
            }
        }

        Err(Error::DoesNotExist)
    }

    ///Disposes of the asset with id
    ///
    ///# Errors
    ///Returns an error if the object with the given id doesn't exist
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
            a.write().dispose();
        }
    }
}

impl Drop for AssetStore {
    // I don't think this is particularly necessary, BUT there may be some custom code in there to
    // ensure something, so i want to call it anyways
    fn drop(&mut self) {
        self.dispose_all();
    }
}
