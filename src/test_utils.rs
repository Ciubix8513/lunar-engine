use futures::executor::block_on;
async fn gen_gpu_async(instance: &wgpu::Instance) -> (wgpu::Device, wgpu::Queue) {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .expect("Unable to get an adapter");

    adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .expect("Can not get device and queue")
}

///Generates all the necessary gpu data for tests
pub(crate) fn generate_gpu() {
    _ = crate::logging::initialize_logging();

    let instance = wgpu::Instance::default();
    let (device, queue) = block_on(gen_gpu_async(&instance));
    #[cfg(not(target_arch = "wasm32"))]
    {
        _ = crate::QUEUE.set(queue);
        _ = crate::DEVICE.set(device);
    }
    #[cfg(target_arch = "wasm32")]
    {
        use crate::wrappers::*;
        _ = crate::QUEUE.set(WgpuWrapper::new(queue));
        _ = crate::DEVICE.set(WgpuWrapper::new(device));
    }
}
