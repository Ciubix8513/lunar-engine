use super::Texture;

pub fn generate_empty_texture() -> Texture {
    Texture::static_png(include_bytes!("../../assets/empty_texture.png"))
}
