use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use lunar_engine_derive::as_any;
use wgpu::util::DeviceExt;

use crate::{
    asset_managment::{Asset, UUID},
    helpers::flip_texture,
};

use lunar_png::Image;

///Stores texture data
pub struct Texture {
    id: Option<UUID>,
    initialized: bool,
    image_format: ImageFormat,
    filepath: Option<PathBuf>,
    r#static: Static,
    mip_count: u8,
    sample_count: u8,
    adress_mode: wgpu::AddressMode,
    filter: wgpu::FilterMode,
    #[cfg(target_arch = "wasm32")]
    pub(crate) sampler: Option<crate::wrappers::WgpuWrapper<wgpu::Sampler>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) sampler: Option<wgpu::Sampler>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) texture: Option<crate::wrappers::WgpuWrapper<wgpu::Texture>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) texture: Option<wgpu::Texture>,
}

///Wether or not the texture is static
enum Static {
    ///Is static, contains the byes of the data
    //Christ, another arc refcell
    Yes(Vec<u8>, Option<Arc<RwLock<Image>>>),
    ///Is not static
    No,
}

///Supported image formats for the asset
enum ImageFormat {
    ///.bmp bitmap(only 32 bpp without compression)
    Bmp,
    ///.png image
    Png,
}

#[allow(unused_variables)]
impl Texture {
    ///Initializes a texture to load a bmp file in runtime
    ///
    ///Currently unsupported on the web target
    ///
    #[must_use]
    pub fn new_bmp(path: &Path) -> Self {
        Self {
            id: None,
            initialized: false,
            image_format: ImageFormat::Bmp,
            filepath: Some(path.to_owned()),
            r#static: Static::No,
            mip_count: 1,
            sample_count: 1,
            adress_mode: wgpu::AddressMode::ClampToEdge,
            filter: wgpu::FilterMode::Linear,
            sampler: None,
            texture: None,
        }
    }

    ///Initializes a texture to parse the texture in runtime, but being loaded at comp time
    ///
    ///Is only supposed to be used for small textures that are always needed
    #[must_use]
    pub fn static_bmp(data: &'static [u8]) -> Self {
        Self {
            id: None,
            initialized: false,
            image_format: ImageFormat::Bmp,
            filepath: None,
            r#static: Static::Yes(data.to_vec(), None),
            mip_count: 1,
            sample_count: 1,
            adress_mode: wgpu::AddressMode::ClampToEdge,
            filter: wgpu::FilterMode::Linear,
            sampler: None,
            texture: None,
        }
    }

    ///Initializes a texture to load a png file in runtime
    ///
    ///Currently unsupported on the web target
    ///
    #[must_use]
    pub fn new_png(path: &Path) -> Self {
        Self {
            id: None,
            initialized: false,
            image_format: ImageFormat::Png,
            filepath: Some(path.to_owned()),
            r#static: Static::No,
            mip_count: 1,
            sample_count: 1,
            adress_mode: wgpu::AddressMode::ClampToEdge,
            filter: wgpu::FilterMode::Linear,
            sampler: None,
            texture: None,
        }
    }

    ///Initializes a texture to parse the texture in runtime, but being loaded at comp time
    ///
    ///Is only supposed to be used for small textures that are always needed
    #[must_use]
    pub fn static_png(data: &'static [u8]) -> Self {
        Self {
            id: None,
            initialized: false,
            image_format: ImageFormat::Png,
            filepath: None,
            r#static: Static::Yes(data.to_vec(), None),
            mip_count: 1,
            sample_count: 1,
            adress_mode: wgpu::AddressMode::ClampToEdge,
            filter: wgpu::FilterMode::Linear,
            sampler: None,
            texture: None,
        }
    }

    /// Loads image data into `wgpu::Texture`
    fn load_into_gpu(&mut self, image: &Arc<RwLock<Image>>) {
        let device = crate::DEVICE.get().unwrap();
        let queue = crate::QUEUE.get().unwrap();
        let image = image.read().unwrap();

        let var_name = &format!("{}", self.get_id());
        let label = Some(var_name.as_str());

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label,
                size: wgpu::Extent3d {
                    width: image.width,
                    height: image.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: self.mip_count.into(),
                sample_count: self.sample_count.into(),
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &image.data,
        );

        drop(image);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label,
            address_mode_u: self.adress_mode,
            address_mode_v: self.adress_mode,
            address_mode_w: self.adress_mode,
            mag_filter: self.filter,
            min_filter: self.filter,
            mipmap_filter: self.filter,
            lod_min_clamp: 1.0,
            lod_max_clamp: 1.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        #[cfg(target_arch = "wasm32")]
        {
            self.texture = Some(crate::wrappers::WgpuWrapper::new(texture));
            self.sampler = Some(crate::wrappers::WgpuWrapper::new(sampler));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.texture = Some(texture);
            self.sampler = Some(sampler);
        }
    }

    // ///Sets the mip count of the texture and generates those mips
    // pub fn set_mip_count(&mut self, count: u8) {
    //     todo!("Not yet implemented")
    // }

    // ///
    // pub fn set_sample_count(&mut self, count: u8) {
    //     todo!("Not yet implemented")
    // }

    ///Sets the filter of the texture
    pub fn set_filter(&mut self, filter: wgpu::FilterMode) {
        //Change the filter and recreate the sampler
        todo!("Not yet implemented")
    }

    ///Sets the address of the texture
    pub fn set_adress_mode(&mut self, filter: wgpu::AddressMode) {
        //Change the address mode and recreate the sampler
        todo!("Not yet implemented")
    }
}

impl Asset for Texture {
    #[as_any]

    fn get_id(&self) -> UUID {
        self.id.unwrap()
    }

    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send>> {
        if let Static::Yes(_, Some(img)) = &self.r#static {
            self.load_into_gpu(&img.clone());
            self.initialized = true;
            return Ok(());
        }

        let image = match &self.r#static {
            Static::Yes(d, _) => d.clone(),
            Static::No => {
                if let Some(file) = &self.filepath {
                    match std::fs::read(file) {
                        Ok(it) => it,
                        Err(err) => return Err(Box::new(err)),
                    }
                } else {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "File not found",
                    )));
                }
            }
        };

        let image = match self.image_format {
            ImageFormat::Bmp => match crate::import::bmp::parse(&image) {
                Ok(it) => it,
                Err(err) => return Err(err),
            },
            ImageFormat::Png => match lunar_png::read_png(&mut image.into_iter()) {
                Ok(mut img) => {
                    flip_texture(&mut img);
                    img.add_alpha();
                    img.add_channels();

                    img
                }
                Err(err) => return Err(Box::new(err)),
            },
        };

        //This is so trash
        let image = Arc::new(RwLock::new(image));
        self.load_into_gpu(&image);

        if let Static::Yes(_, arc) = &mut self.r#static {
            *arc = Some(image);
        }
        self.initialized = true;

        Ok(())
    }

    fn dispose(&mut self) {
        //Don't dispose of static textures
        if let Static::Yes(..) = self.r#static {
            return;
        }
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
