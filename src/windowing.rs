use std::sync::RwLock;

use vec_key_value_pair::VecMap;
use wgpu::{util::StagingBelt, Surface, SurfaceConfiguration, Texture};

use crate::{input::InputState, math::vec2::Vec2, DEVICE, FORMAT, QUEUE, RESOLUTION, STAGING_BELT};

pub fn initialize_logging() {
    env_logger::Builder::new()
        // .filter_module("wgpu", log::LevelFilter::Info) .filter_module("lunar_engine", log::LevelFilter::Info)
        .filter_level(log::LevelFilter::Info)
        .init();
}

pub fn initialize_gpu(window: &winit::window::Window) -> (Surface, SurfaceConfiguration, Texture) {
    let size = window.inner_size();
    *RESOLUTION.write().unwrap() = size;

    let instance = wgpu::Instance::default();

    let surface = unsafe {
        instance
            .create_surface(&window)
            .expect("Failed to createate surface")
    };

    let adapter: wgpu::Adapter = futures::executor::block_on(req_adapter(
        instance,
        &wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        },
    ))
    .expect("Failed to get an adapter");

    let (device, queue): (wgpu::Device, wgpu::Queue) = futures::executor::block_on(req_device(
        &adapter,
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::DEPTH_CLIP_CONTROL,
            ..Default::default()
        },
    ))
    .expect("Failed to create a device and a queue");

    DEVICE.set(device).unwrap();
    QUEUE.set(queue).unwrap();

    let device = DEVICE.get().unwrap();

    let capabilities = surface.get_capabilities(&adapter);
    let format = capabilities
        .formats
        .last()
        .copied()
        .expect("Did not have last format");
    FORMAT.set(format).unwrap();
    assert!(
        capabilities.usages & wgpu::TextureUsages::RENDER_ATTACHMENT
            == wgpu::TextureUsages::RENDER_ATTACHMENT,
        "Rendering not supported... What shitty ancient piece of shit are you fucking using wtf?"
    );

    let surface_config = wgpu::SurfaceConfiguration {
        usage: if capabilities.usages & wgpu::TextureUsages::COPY_SRC
            == wgpu::TextureUsages::COPY_SRC
        {
            // features.screenshot = true;
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC
        } else {
            log::warn!("Screenshot feature not supported!");
            wgpu::TextureUsages::RENDER_ATTACHMENT
        },
        format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoNoVsync,
        view_formats: vec![format],
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
    };
    surface.configure(device, &surface_config);

    let desc = get_depth_descriptor(size.width, size.height);
    let depth_stencil = device.create_texture(&desc);

    let belt = StagingBelt::new(2048);

    STAGING_BELT.set(RwLock::new(belt)).unwrap();

    // let bpr = helpers::calculate_bpr(size.width, format);
    // let screenshot_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    //     label: Some("Screenshot buffer"),
    //     size: bpr * u64::from(size.height),
    //     usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    //     mapped_at_creation: false,
    // });

    super::input::INPUT
        .set(InputState {
            key_map: RwLock::new(VecMap::new()),
            mouse_button_map: RwLock::new(VecMap::new()),
            cursor_position: RwLock::new(Vec2::default()),
            previous_cursor_position: RwLock::new(Vec2::default()),
            cursor_delta: RwLock::new(Vec2::default()),
        })
        .unwrap();

    (surface, surface_config, depth_stencil)
}

async fn req_adapter<'a>(
    instance: wgpu::Instance,
    options: &wgpu::RequestAdapterOptions<'a>,
) -> Option<wgpu::Adapter> {
    instance.request_adapter(options).await
}

async fn req_device<'a>(
    adapter: &wgpu::Adapter,
    descriptor: &wgpu::DeviceDescriptor<'a>,
) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
    adapter.request_device(descriptor, None).await
}

pub(crate) fn get_depth_descriptor<'a>(width: u32, height: u32) -> wgpu::TextureDescriptor<'a> {
    wgpu::TextureDescriptor {
        label: Some("Depth stencil"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[wgpu::TextureFormat::Depth32Float],
    }
}
