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
        let extracted = self.extractor.extract(headers).await?;

        let mapped = (self.map)(extracted);

        Ok(mapped)
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
