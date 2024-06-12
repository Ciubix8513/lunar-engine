use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use wgpu::util::DeviceExt;

use crate::{
    asset_managment::{Asset, AssetStore, UUID},
    structures::Image,
    DEVICE,
};

#[cfg(test)]
mod tests;

///Contains implemented materials
pub mod materials;

//============================================================
//===========================Texture==========================
//============================================================

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
    sampler: Option<crate::wrappers::WgpuWrapper<wgpu::Sampler>>,
    #[cfg(not(target_arch = "wasm32"))]
    sampler: Option<wgpu::Sampler>,
    #[cfg(target_arch = "wasm32")]
    texture: Option<crate::wrappers::WgpuWrapper<wgpu::Texture>>,
    #[cfg(not(target_arch = "wasm32"))]
    texture: Option<wgpu::Texture>,
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
            bytemuck::cast_slice(&image.data),
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

    pub fn set_mip_count(&mut self, count: u8) {
        todo!("Not yet implemented")
    }

    pub fn set_sample_count(&mut self, count: u8) {
        todo!("Not yet implemented")
    }

    pub fn set_filter(&mut self, filter: wgpu::FilterMode) {
        todo!("Not yet implemented")
    }

    pub fn set_adress_mode(&mut self, filter: wgpu::AddressMode) {
        todo!("Not yet implemented")
    }
}

impl Asset for Texture {
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

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

//============================================================
//===========================Mesh=============================
//============================================================
//Yea this trash is not expandale later on, gltf may be a bit too complex for loading into
//separate assets, who knows tho
//
//Actually, thinking about it, i think loading a gltf should produce an entire set of assets

///Asset that stores mesh data
pub struct Mesh {
    id: Option<UUID>,
    initialized: bool,
    mode: MeshMode,
    #[cfg(target_arch = "wasm32")]
    vertex_buffer: Option<Arc<crate::wrappers::WgpuWrapper<wgpu::Buffer>>>,
    #[cfg(target_arch = "wasm32")]
    index_buffer: Option<Arc<crate::wrappers::WgpuWrapper<wgpu::Buffer>>>,
    #[cfg(not(target_arch = "wasm32"))]
    vertex_buffer: Option<Arc<wgpu::Buffer>>,
    #[cfg(not(target_arch = "wasm32"))]
    index_buffer: Option<Arc<wgpu::Buffer>>,
    vert_count: Option<u32>,
    tris_count: Option<u32>,
    index_count: Option<u32>,
}

///Ways the mesh can be loaded from file
enum MeshMode {
    ///An obj file that contains a single mesh
    SingleObjectOBJ(PathBuf),
    StaticSingleObjectOBJ(&'static str),
}

impl Mesh {
    ///Creates anew asset that will load the first object in a waveform obj file that is statically
    ///loaded
    #[must_use]
    pub const fn new_from_static_obj(mesh: &'static str) -> Self {
        Self {
            id: None,
            initialized: false,
            mode: MeshMode::StaticSingleObjectOBJ(mesh),
            vertex_buffer: None,
            index_buffer: None,
            vert_count: None,
            tris_count: None,
            index_count: None,
        }
    }

    ///Creates a new asset that will load the first object in a waveform obj file
    ///
    ///Currently unsupported on the web target
    ///
    ///# Errors
    ///Returns an error if the file does not exist
    pub fn new_from_obj(path: &Path) -> Result<Self, std::io::Error> {
        //Verify that file exists
        std::fs::File::options().read(true).open(path)?;
        Ok(Self {
            id: None,
            initialized: false,
            mode: MeshMode::SingleObjectOBJ(path.to_owned()),
            vertex_buffer: None,
            index_buffer: None,
            tris_count: None,
            vert_count: None,
            index_count: None,
        })
    }

    ///Returns the vertex buffer of the mesh
    ///
    ///# Panics
    ///Panics if the asset was not initialized
    #[cfg(target_arch = "wasm32")]
    #[must_use]
    pub fn get_vertex_buffer(&self) -> Arc<crate::wrappers::WgpuWrapper<wgpu::Buffer>> {
        //THIS IS SO TRASH
        self.vertex_buffer.clone().unwrap()
    }

    ///Returns the vertex buffer of the mesh
    ///
    ///# Panics
    ///Panics if the asset was not initialized
    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    pub fn get_vertex_buffer(&self) -> Arc<wgpu::Buffer> {
        self.vertex_buffer.clone().unwrap()
    }

    ///Returns the index buffer of the mesh
    ///
    ///# Panics
    ///Panics if the asset was not initialized
    #[cfg(target_arch = "wasm32")]
    #[must_use]
    pub fn get_index_buffer(&self) -> Arc<crate::wrappers::WgpuWrapper<wgpu::Buffer>> {
        self.index_buffer.clone().unwrap()
    }

    ///Returns the index buffer of the mesh
    ///
    ///# Panics
    ///Panics if the asset was not initialized
    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    pub fn get_index_buffer(&self) -> Arc<wgpu::Buffer> {
        self.index_buffer.clone().unwrap()
    }

    ///Returns the vertex count of the mesh
    ///
    ///# Panics
    ///Panics if the asset was not initialized
    #[must_use]
    pub fn get_tris_count(&self) -> u32 {
        self.tris_count.unwrap()
    }

    ///Returns the index count of the mesh
    ///
    ///# Panics
    ///Panics if the asset was not initialized
    #[must_use]
    pub fn get_index_count(&self) -> u32 {
        self.index_count.unwrap()
    }

    ///Returns the vertex count of the mesh
    ///
    ///# Panics
    ///Panics if the asset was not initialized
    #[must_use]
    pub fn get_vert_count(&self) -> u32 {
        self.vert_count.unwrap()
    }
}

impl Asset for Mesh {
    fn get_id(&self) -> UUID {
        self.id.unwrap()
    }

    #[allow(clippy::cast_possible_truncation)]
    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send>> {
        //This is horrific, but i LOVE this :3
        let mesh = match &self.mode {
            MeshMode::SingleObjectOBJ(path) => {
                //Prase file
                match crate::import::obj::parse(
                    //Load file
                    &(match std::fs::read_to_string(path) {
                        Ok(it) => it,
                        Err(err) => return Err(Box::new(err)),
                    }),
                )
                //Get the first mesh
                .and_then(|i| i.into_iter().nth(0))
                {
                    Some(it) => it,
                    None => {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Invalid file",
                        )));
                    }
                }
            }
            MeshMode::StaticSingleObjectOBJ(mesh) => {
                match crate::import::obj::parse(mesh).and_then(|i| i.into_iter().nth(0)) {
                    Some(it) => it,
                    None => {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Invalid data",
                        )));
                    }
                }
            }
        };

        let device = DEVICE.get().unwrap();
        let name = format!("Mesh {}", self.get_id());

        let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&name),
            contents: bytemuck::cast_slice(mesh.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&name),
            contents: bytemuck::cast_slice(mesh.indecies.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });

        #[cfg(target_arch = "wasm32")]
        {
            self.vertex_buffer = Some(Arc::new(crate::wrappers::WgpuWrapper::new(vb)));
            self.index_buffer = Some(Arc::new(crate::wrappers::WgpuWrapper::new(ib)));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.vertex_buffer = Some(Arc::new(vb));
            self.index_buffer = Some(Arc::new(ib));
        }
        self.vert_count = Some(mesh.vertices.len() as u32);
        self.tris_count = Some((mesh.indecies.len() as u32) / 3u32);
        self.index_count = Some(mesh.indecies.len() as u32);

        self.initialized = true;
        Ok(())
    }

    fn dispose(&mut self) {
        //Unload index and vertex buffers, clearing memory
        self.vertex_buffer = None;
        self.index_buffer = None;
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

//============================================================
//===========================MATERIAL==========================
//============================================================

#[derive(Clone, Copy)]
pub enum BindgroupState {
    Uninitialized,
    Initialized,
}

pub trait MaterialTrait {
    fn render(&self, render_pass: &mut wgpu::RenderPass);
    fn intialize(&mut self);
    fn dispose(&mut self);
    fn set_bindgroups(&mut self, asset_store: &AssetStore);
    fn bindgroup_sate(&self) -> BindgroupState;
}

///Stores material data
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
    pub fn get_bindgroup_state(&self) -> BindgroupState {
        self.material.bindgroup_sate()
    }

    pub fn initialize_bindgroups(&mut self, asset_store: &AssetStore) {
        self.material.set_bindgroups(asset_store);
    }

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
