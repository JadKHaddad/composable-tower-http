use std::future::Future;

use http::HeaderMap;

use crate::{chain::chain_extractor::ChainExtractor, error::InfallibleError};

use super::any::Any;

pub trait Extractor {
    type Extracted: Clone + Send + Sync;

    type Error;

    fn extract(
        &self,
        headers: &HeaderMap,
    ) -> impl Future<Output = Result<Self::Extracted, Self::Error>> + Send;
}

pub trait ExtractorExt: Sized + Extractor {
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

    fn chain<C>(self, chain: C) -> ChainExtractor<Self, C> {
        ChainExtractor::new(self, chain)
    }

    fn chain_lite<Fn>(self, chain: Fn) -> ChainLite<Self, Fn> {
        ChainLite::new(self, chain)
    }

    fn async_chain_lite<Fn>(self, chain: Fn) -> AsyncChainLite<Self, Fn> {
        AsyncChainLite::new(self, chain)
    }

    fn optional(self) -> Optional<Self> {
        Optional::new(self)
    }

    fn any<Ex>(self, other: Ex) -> Any<Self, Ex> {
        Any::new(self, other)
    }
}

impl<T> ExtractorExt for T where T: Sized + Extractor {}

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

#[derive(Debug, Clone)]
pub struct ChainLite<T, Fn> {
    inner: T,
    chain: Fn,
}

impl<T, Fn> ChainLite<T, Fn> {
    pub const fn new(inner: T, chain: Fn) -> Self {
        Self { inner, chain }
    }
}

#[derive(Debug, Clone)]
pub struct AsyncChainLite<T, Fn> {
    inner: T,
    chain: Fn,
}

impl<T, Fn> AsyncChainLite<T, Fn> {
    pub const fn new(inner: T, chain: Fn) -> Self {
        Self { inner, chain }
    }
}

#[derive(Debug, Clone)]
pub struct Optional<T> {
    inner: T,
}

impl<T> Optional<T> {
    pub const fn new(inner: T) -> Self {
        Self { inner }
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

impl<Ex, Fn, T, E> Extractor for ChainLite<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Extracted) -> Result<T, E> + Copy + Sync,
    T: Clone + Send + Sync + 'static,
    E: From<Ex::Error>,
{
    type Extracted = T;

    type Error = E;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let ex = self.inner.extract(headers).await?;

        (self.chain)(ex)
    }
}

impl<Ex, Fn, T, E, Fut> Extractor for AsyncChainLite<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Extracted) -> Fut + Copy + Sync,
    Fut: Future<Output = Result<T, E>> + Send,
    T: Clone + Send + Sync + 'static,
    E: From<Ex::Error>,
{
    type Extracted = T;

    type Error = E;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let ex = self.inner.extract(headers).await?;

        (self.chain)(ex).await
    }
}

impl<Ex> Extractor for Optional<Ex>
where
    Ex: Extractor + Sync,
{
    type Extracted = Option<Ex::Extracted>;

    type Error = InfallibleError;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        Ok(self.inner.extract(headers).await.ok())
    }
}
