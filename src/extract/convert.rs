use std::future::Future;

use http::HeaderMap;

use super::extractor::Extractor;

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

impl<Ex, Fn, T, E> Extractor for Convert<Ex, Fn>
where
    Ex: Extractor + Sync,
    Fn: FnOnce(Result<Ex::Extracted, Ex::Error>) -> Result<T, E> + Copy + Sync,
    T: Clone + Send + Sync,
{
    type Extracted = T;

    type Error = E;

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

    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let ex = self.inner.extract(headers).await;

        (self.convert)(ex).await
    }
}
