//! Implemented assets

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
pub enum BindgroupState {
    Uninitialized,
    Initialized,
}
