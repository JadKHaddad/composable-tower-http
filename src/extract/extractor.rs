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
    fn map<Fn>(self, map: Fn) -> Map<Self, Fn>;

    fn async_map<Fn>(self, map: Fn) -> AsyncMap<Self, Fn>;

    fn map_err<Fn>(self, map_err: Fn) -> ErrorMap<Self, Fn>;

    fn convert<Fn>(self, convert: Fn) -> Convert<Self, Fn>;

    fn async_convert<Fn>(self, convert: Fn) -> AsyncConvert<Self, Fn>;

    fn validated<V>(self, validator: V) -> ValidationExtractor<Self, V>;
}

impl<T> ExtractorExt for T
where
    T: Sized + Extractor,
{
    fn map<Fn>(self, map: Fn) -> Map<Self, Fn> {
        Map::new(self, map)
    }

    fn async_map<Fn>(self, map: Fn) -> AsyncMap<Self, Fn> {
        AsyncMap::new(self, map)
    }

    fn map_err<Fn>(self, map_err: Fn) -> ErrorMap<Self, Fn> {
        ErrorMap::new(self, map_err)
    }

    fn convert<Fn>(self, convert: Fn) -> Convert<Self, Fn> {
        Convert::new(self, convert)
    }

    fn async_convert<Fn>(self, convert: Fn) -> AsyncConvert<Self, Fn> {
        AsyncConvert::new(self, convert)
    }

    fn validated<V>(self, validator: V) -> ValidationExtractor<Self, V> {
        ValidationExtractor::new(self, validator)
    }
}

#[derive(Debug, Clone)]
pub struct Map<T, Fn> {
    inner: T,
    map: Fn,
}

impl<T, Fn> Map<T, Fn> {
    pub const fn new(inner: T, map: Fn) -> Self {
        Self { inner, map }
    }
}

#[derive(Debug, Clone)]
pub struct AsyncMap<T, Fn> {
    inner: T,
    map: Fn,
}

impl<T, Fn> AsyncMap<T, Fn> {
    pub const fn new(inner: T, map: Fn) -> Self {
        Self { inner, map }
    }
}

#[derive(Debug, Clone)]
pub struct ErrorMap<T, Fn> {
    inner: T,
    map_err: Fn,
}

impl<T, Fn> ErrorMap<T, Fn> {
    pub const fn new(inner: T, map_err: Fn) -> Self {
        Self { inner, map_err }
    }
}

#[derive(Debug, Clone)]
pub struct Convert<T, Fn> {
    inner: T,
    convert: Fn,
}

impl<T, Fn> Convert<T, Fn> {
    pub const fn new(inner: T, convert: Fn) -> Self {
        Self { inner, convert }
    }
}

#[derive(Debug, Clone)]
pub struct AsyncConvert<T, Fn> {
    inner: T,
    convert: Fn,
}

impl<T, Fn> AsyncConvert<T, Fn> {
    pub const fn new(inner: T, convert: Fn) -> Self {
        Self { inner, convert }
    }
}

impl<Ex, Fn, T> Extractor for Map<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Extracted) -> T + Copy + Sync,
    T: Clone + Send + Sync,
{
    type Extracted = T;

    type Error = Ex::Error;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        self.inner.extract(headers).await.map(|ex| (self.map)(ex))
    }
}

impl<Ex, Fn, T, Fut> Extractor for AsyncMap<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Extracted) -> Fut + Copy + Sync,
    Fut: Future<Output = T> + Send,
    T: Clone + Send + Sync,
{
    type Extracted = T;

    type Error = Ex::Error;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let extracted = self.inner.extract(headers).await?;

        let mapped = (self.map)(extracted).await;

        Ok(mapped)
    }
}

impl<Ex, Fn, E> Extractor for ErrorMap<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Error) -> E + Copy + Sync,
{
    type Extracted = Ex::Extracted;

    type Error = E;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        self.inner
            .extract(headers)
            .await
            .map_err(|err| (self.map_err)(err))
    }
}

impl<Ex, Fn, T, E> Extractor for Convert<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Result<Ex::Extracted, Ex::Error>) -> Result<T, E> + Copy + Sync,
    T: Clone + Send + Sync,
{
    type Extracted = T;

    type Error = E;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let ex = self.inner.extract(headers).await;

        (self.convert)(ex)
    }
}

impl<Ex, Fn, T, E, Fut> Extractor for AsyncConvert<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Result<Ex::Extracted, Ex::Error>) -> Fut + Copy + Sync,
    Fut: Future<Output = Result<T, E>> + Send,
    T: Clone + Send + Sync,
{
    type Extracted = T;

    type Error = E;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let ex = self.inner.extract(headers).await;

        (self.convert)(ex).await
    }
}
