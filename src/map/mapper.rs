use std::future::Future;

use crate::extract::extractor::Extractor;

#[derive(Debug, Clone)]
pub struct Mapper<Ex, Fn> {
    extractor: Ex,
    map: Fn,
}

impl<Ex, Fn> Mapper<Ex, Fn> {
    pub const fn new(extractor: Ex, map: Fn) -> Self {
        Self { extractor, map }
    }
}

impl<Ex, Fn, T> Extractor for Mapper<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Extracted) -> T + Copy + Sync,
    T: Clone + Send + Sync,
{
    type Extracted = T;

    type Error = Ex::Error;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        self.extractor
            .extract(headers)
            .await
            .map(|ex| (self.map)(ex))
    }
}

#[derive(Debug, Clone)]
pub struct AsyncMapper<Ex, Fn> {
    extractor: Ex,
    map: Fn,
}

impl<Ex, Fn> AsyncMapper<Ex, Fn> {
    pub const fn new(extractor: Ex, map: Fn) -> Self {
        Self { extractor, map }
    }
}

impl<Ex, Fn, T, Fut> Extractor for AsyncMapper<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Extracted) -> Fut + Copy + Sync,
    Fut: Future<Output = T> + Send,
    T: Clone + Send + Sync,
{
    type Extracted = T;

    type Error = Ex::Error;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let extracted = self.extractor.extract(headers).await?;

        let mapped = (self.map)(extracted).await;

        Ok(mapped)
    }
}

#[derive(Debug, Clone)]
pub struct ErrorMapper<Ex, Fn> {
    extractor: Ex,
    map_err: Fn,
}

impl<Ex, Fn> ErrorMapper<Ex, Fn> {
    pub const fn new(extractor: Ex, map_err: Fn) -> Self {
        Self { extractor, map_err }
    }
}

impl<Ex, Fn, E> Extractor for ErrorMapper<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Error) -> E + Copy + Sync,
    E: Clone + Send + Sync,
{
    type Extracted = Ex::Extracted;

    type Error = E;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        self.extractor
            .extract(headers)
            .await
            .map_err(|err| (self.map_err)(err))
    }
}

pub trait MapperExt: Sized + Extractor {
    fn map<Fn>(self, map: Fn) -> Mapper<Self, Fn>;

    fn async_map<Fn>(self, map: Fn) -> AsyncMapper<Self, Fn>;

    fn map_err<Fn>(self, map_err: Fn) -> ErrorMapper<Self, Fn>;
}

impl<T> MapperExt for T
where
    T: Sized + Extractor,
{
    fn map<Fn>(self, map: Fn) -> Mapper<Self, Fn> {
        Mapper::new(self, map)
    }

    fn async_map<Fn>(self, map: Fn) -> AsyncMapper<Self, Fn> {
        AsyncMapper::new(self, map)
    }

    fn map_err<Fn>(self, map_err: Fn) -> ErrorMapper<Self, Fn> {
        ErrorMapper::new(self, map_err)
    }
}
