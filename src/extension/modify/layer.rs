use std::marker::PhantomData;

use tower::Layer;

use super::service::ModificationService;

#[derive(Debug, Clone)]
pub struct ModificationLayer<M, T> {
    modifier: M,
    _phantom: PhantomData<T>,
}

impl<M, T> ModificationLayer<M, T> {
    pub const fn new(modifier: M) -> Self {
        Self {
            modifier,
            _phantom: PhantomData,
        }
    }
}

impl<S, M, T> Layer<S> for ModificationLayer<M, T>
where
    M: Clone,
{
    type Service = ModificationService<S, M, T>;

    fn layer(&self, service: S) -> Self::Service {
        ModificationService::new(service, self.modifier.clone())
    }
}

pub trait ModificationLayerExt: Sized {
    fn modification_layer<T>(self) -> ModificationLayer<Self, T>;
}

impl<T> ModificationLayerExt for T
where
    T: Sized + Clone,
{
    fn modification_layer<M>(self) -> ModificationLayer<Self, M> {
        ModificationLayer::new(self)
    }
}
