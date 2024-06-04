use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct WgpuWrapper<T>(send_wrapper::SendWrapper<T>);

impl<T> Deref for WgpuWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for WgpuWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

unsafe impl<T> Send for WgpuWrapper<T> {}
unsafe impl<T> Sync for WgpuWrapper<T> {}

impl<T> WgpuWrapper<T> {
    pub fn new(t: T) -> Self {
        Self(send_wrapper::SendWrapper::new(t))
    }
}
