//============================================================
//===========================Mesh=============================
//============================================================
//Yea this trash is not expandale later on, gltf may be a bit too complex for loading into
//separate assets, who knows tho
//
//Actually, thinking about it, i think loading a gltf should produce an entire set of assets

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use lunar_engine_derive::as_any;
use wgpu::util::DeviceExt;

use crate::{
    asset_managment::{Asset, UUID},
    DEVICE,
};

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
    #[as_any]

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
}
