#![allow(clippy::too_many_lines)]
use std::sync::RwLock;

use vec_key_value_pair::map::VecMap;
use wgpu::{Backends, Surface, SurfaceConfiguration, Texture, util::StagingBelt};
use winit::window::Window;

use crate::{
    APP_INFO, DEVICE, FORMAT, QUEUE, RESOLUTION, STAGING_BELT, input::InputState, math::Vec2,
};

pub fn initialize_gpu(window: &Window) -> (Surface<'_>, SurfaceConfiguration, Texture) {
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.width.max(1);
    log::debug!("Window size is {size:?}");
    *RESOLUTION.write().unwrap() = size;

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::GL | Backends::VULKAN,
        ..Default::default()
    });

    let surface = instance
        .create_surface(window)
        .expect("Failed to createate surface");

    log::debug!("Created surface");

    let adapter: wgpu::Adapter = futures::executor::block_on(req_adapter(
        instance,
        &wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        },
    ))
    .expect("Failed to get an adapter");

    log::debug!("Acquired an adapter");

    #[cfg(feature = "webgl")]
    let limits = wgpu::Limits::downlevel_webgl2_defaults();

    #[cfg(not(feature = "webgl"))]
    let limits = wgpu::Limits::default();

    let (device, queue): (wgpu::Device, wgpu::Queue) = {
        let r = futures::executor::block_on(req_device(
            &adapter,
            &wgpu::DeviceDescriptor {
                required_limits: limits,
                ..Default::default()
            },
        ));
        if let Err(e) = r {
            log::error!("Error while getting device {e}");
            panic!();
        }
        r.unwrap()
    };
    log::debug!("Created device and queue");

    DEVICE.set(device).unwrap();
    QUEUE.set(queue).unwrap();

    let device = DEVICE.get().unwrap();

    let capabilities = surface.get_capabilities(&adapter);
    let format = capabilities
        .formats
        .last()
        .copied()
        .expect("Did not have last format");

    log::debug!("Picked a format");

    FORMAT.set(format).unwrap();
    assert!(
        capabilities.usages & wgpu::TextureUsages::RENDER_ATTACHMENT
            == wgpu::TextureUsages::RENDER_ATTACHMENT,
        "Rendering not supported... What shitty ancient piece of shit are you fucking using wtf?"
    );

    let screenshot_supported =
        capabilities.usages & wgpu::TextureUsages::COPY_SRC == wgpu::TextureUsages::COPY_SRC;

    APP_INFO
        .get()
        .unwrap()
        .write()
        .unwrap()
        .screenshot_supported = screenshot_supported;

    let surface_config = wgpu::SurfaceConfiguration {
        usage: if screenshot_supported {
            // features.screenshot = true;
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC
        } else {
            log::warn!("Screenshot feature not supported!");
            wgpu::TextureUsages::RENDER_ATTACHMENT
        },
        format,
        width: size.width,
        height: size.height,
        view_formats: vec![format],
        desired_maximum_frame_latency: 2,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,

        present_mode: wgpu::PresentMode::AutoNoVsync,
    };
    surface.configure(device, &surface_config);

    log::debug!("Configured the surface");

    let desc = get_depth_descriptor(size.width, size.height);
    let depth_stencil = device.create_texture(&desc);

    log::debug!("Created depth texture");

    let belt = StagingBelt::new(4096);

    log::debug!("Created staging belt");

    STAGING_BELT.set(RwLock::new(belt)).unwrap();

    super::input::INPUT
        .set(InputState {
            key_map: RwLock::new(VecMap::new()),
            mouse_button_map: RwLock::new(VecMap::new()),
            cursor_position: RwLock::new(Vec2::default()),
            previous_cursor_position: RwLock::new(Vec2::default()),
            cursor_delta: RwLock::new(Vec2::default()),
            raw_curosor_delta: RwLock::new(Vec2::default()),
            delta_changed: RwLock::new(false),
        })
        .unwrap();

    (surface, surface_config, depth_stencil)
}

#[allow(clippy::future_not_send)]
async fn req_adapter(
    instance: wgpu::Instance,
    options: &wgpu::RequestAdapterOptions<'_, '_>,
) -> Result<wgpu::Adapter, wgpu::RequestAdapterError> {
    instance.request_adapter(options).await
}

#[allow(clippy::future_not_send)]
async fn req_device(
    adapter: &wgpu::Adapter,
    descriptor: &wgpu::DeviceDescriptor<'_>,
) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
    adapter.request_device(descriptor).await
}

pub fn get_depth_descriptor<'a>(width: u32, height: u32) -> wgpu::TextureDescriptor<'a> {
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
