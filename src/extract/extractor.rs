use std::future::Future;

use http::HeaderMap;

use crate::validate::extract::validation_extractor::ValidationExtractor;

pub trait Extractor {
    type Extracted: Clone + Send + Sync;

    type Error;

    fn extract(
        &self,
        headers: &HeaderMap,
    ) -> impl Future<Output = Result<Self::Extracted, Self::Error>> + Send;
}

pub trait ExtractorExt: Sized + Extractor {
    fn map<Fn>(self, map: Fn) -> Mapper<Self, Fn>;

    fn async_map<Fn>(self, map: Fn) -> AsyncMapper<Self, Fn>;

    fn map_err<Fn>(self, map_err: Fn) -> ErrorMapper<Self, Fn>;

    fn validated<V>(self, validator: V) -> ValidationExtractor<Self, V>;
}

impl<T> ExtractorExt for T
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

    fn validated<V>(self, validator: V) -> ValidationExtractor<Self, V> {
        ValidationExtractor::new(self, validator)
    }
}

#[derive(Debug, Clone)]
pub struct Mapper<T, Fn> {
    inner: T,
    map: Fn,
}

impl<T, Fn> Mapper<T, Fn> {
    pub const fn new(inner: T, map: Fn) -> Self {
        Self { inner, map }
    }
}

#[derive(Debug, Clone)]
pub struct AsyncMapper<T, Fn> {
    inner: T,
    map: Fn,
}

impl<T, Fn> AsyncMapper<T, Fn> {
    pub const fn new(inner: T, map: Fn) -> Self {
        Self { inner, map }
    }
}

#[derive(Debug, Clone)]
pub struct ErrorMapper<T, Fn> {
    inner: T,
    map_err: Fn,
}

impl<T, Fn> ErrorMapper<T, Fn> {
    pub const fn new(inner: T, map_err: Fn) -> Self {
        Self { inner, map_err }
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
        self.inner.extract(headers).await.map(|ex| (self.map)(ex))
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
        let extracted = self.inner.extract(headers).await?;

        let mapped = (self.map)(extracted).await;

        Ok(mapped)
    }
}

impl<Ex, Fn, E> Extractor for ErrorMapper<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Error) -> E + Copy + Sync,
{
    type Extracted = Ex::Extracted;

    type Error = E;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        self.inner
            .extract(headers)
            .await
            .map_err(|err| (self.map_err)(err))
    }
}
