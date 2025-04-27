//! Implemented assets

pub(crate) mod heleprs;
///Material struct
pub mod material;
///Contains implemented materials
pub mod materials;
///Mesh asset
pub mod mesh;
#[cfg(test)]
mod tests;
///Texture asset
pub mod texture;

pub use material::Material;
pub use mesh::Mesh;
pub use texture::Texture;

#[derive(Clone, Copy)]
///Represents bindroup state of an asset that contains gpu related data
pub enum BindgroupState {
    ///Bindgroups are not initialized
    Uninitialized,
    ///Bindgroups are initialized
    Initialized,
}
