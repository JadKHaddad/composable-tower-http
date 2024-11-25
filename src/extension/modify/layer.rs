use std::marker::PhantomData;

use tower::Layer;

use super::service::ModifyService;

#[derive(Debug, Clone)]
pub struct ModifyLayer<M, T> {
    modifier: M,
    _phantom: PhantomData<T>,
}

impl<M, T> ModifyLayer<M, T> {
    pub const fn new(modifier: M) -> Self {
        Self {
            modifier,
            _phantom: PhantomData,
        }
    }
}

impl<S, M, T> Layer<S> for ModifyLayer<M, T>
where
    M: Clone,
{
    type Service = ModifyService<S, M, T>;

    fn layer(&self, service: S) -> Self::Service {
        ModifyService::new(service, self.modifier.clone())
    }
}

pub trait ModifyLayerExt<T>: Sized {
    fn layer(self) -> ModifyLayer<Self, T>;
}

impl<T, M> ModifyLayerExt<M> for T
where
    T: Sized + Clone,
{
    fn layer(self) -> ModifyLayer<Self, M> {
        ModifyLayer::new(self)
    }
}
