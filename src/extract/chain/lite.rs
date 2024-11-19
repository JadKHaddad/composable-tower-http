use std::future::Future;

use http::HeaderMap;

use crate::extract::extractor::Extractor;

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

impl<Ex, Fn, T, E> Extractor for ChainLite<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Extracted) -> Result<T, E> + Clone + Sync,
    T: Clone + Send + Sync + 'static,
    E: From<Ex::Error>,
{
    type Extracted = T;

    type Error = E;

    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let ex = self.inner.extract(headers).await?;

        (self.chain.clone())(ex)
    }
}

impl<Ex, Fn, T, E, Fut> Extractor for AsyncChainLite<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Ex::Extracted) -> Fut + Clone + Sync,
    Fut: Future<Output = Result<T, E>> + Send,
    T: Clone + Send + Sync + 'static,
    E: From<Ex::Error>,
{
    type Extracted = T;

    type Error = E;

    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let ex = self.inner.extract(headers).await?;

        (self.chain.clone())(ex).await
    }
}
