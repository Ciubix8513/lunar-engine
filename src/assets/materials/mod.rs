pub use color_lit::ColorLit;
pub use color_unlit::ColorUnlit;
pub use lit::Lit;
pub use texture_unlit::TextureUnlit;

mod color_lit;
mod color_unlit;
mod lit;
mod texture_unlit;

///Helper functions for implementing materials
pub mod helpers;
