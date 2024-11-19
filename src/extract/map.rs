use std::future::Future;

use http::HeaderMap;

use super::extractor::Extractor;

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
pub struct MapError<T, Fn> {
    inner: T,
    map_err: Fn,
}

impl<T, Fn> MapError<T, Fn> {
    pub const fn new(inner: T, map_err: Fn) -> Self {
        Self { inner, map_err }
    }
}

impl<Ex, Fn, T> Extractor for Map<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Extracted) -> T + Clone + Sync,
    T: Clone + Send + Sync,
{
    type Extracted = T;

    type Error = Ex::Error;

    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        self.inner
            .extract(headers)
            .await
            .map(|ex| (self.map.clone())(ex))
    }
}

impl<Ex, Fn, T, Fut> Extractor for AsyncMap<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Extracted) -> Fut + Clone + Sync,
    Fut: Future<Output = T> + Send,
    T: Clone + Send + Sync,
{
    type Extracted = T;

    type Error = Ex::Error;

    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let extracted = self.inner.extract(headers).await?;

        let mapped = (self.map.clone())(extracted).await;

        Ok(mapped)
    }
}

impl<Ex, Fn, E> Extractor for MapError<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Error) -> E + Clone + Sync,
{
    type Extracted = Ex::Extracted;

    type Error = E;

    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        self.inner
            .extract(headers)
            .await
            .map_err(|err| (self.map_err.clone())(err))
    }
}
