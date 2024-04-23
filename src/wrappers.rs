#[cfg(target_arch = "wasm32")]
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct WgpuWrapper<T>(send_wrapper::SendWrapper<T>);

#[cfg(target_arch = "wasm32")]
impl<T> Deref for WgpuWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(target_arch = "wasm32")]
impl<T> DerefMut for WgpuWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(target_arch = "wasm32")]
unsafe impl<T> Send for WgpuWrapper<T> {}
#[cfg(target_arch = "wasm32")]
unsafe impl<T> Sync for WgpuWrapper<T> {}

#[cfg(target_arch = "wasm32")]
impl<T> WgpuWrapper<T> {
    pub fn new(t: T) -> Self {
        Self(send_wrapper::SendWrapper::new(t))
    }
}
