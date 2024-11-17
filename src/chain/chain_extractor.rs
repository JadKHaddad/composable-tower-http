use std::{ops::Deref, sync::Arc};

use crate::extract::extractor::Extractor;

use super::chainer::Chainer;

#[derive(Debug)]
pub struct ChainExtractorInner<Ex, C> {
    extractor: Ex,
    chainer: C,
}

impl<Ex, C> ChainExtractorInner<Ex, C> {
    pub const fn new(extractor: Ex, chainer: C) -> Self {
        Self { extractor, chainer }
    }
}

#[derive(Debug)]
pub struct ChainExtractor<Ex, C> {
    inner: Arc<ChainExtractorInner<Ex, C>>,
}

impl<Ex, C> ChainExtractor<Ex, C> {
    pub fn new(extractor: Ex, chainer: C) -> Self {
        Self {
            inner: Arc::new(ChainExtractorInner::new(extractor, chainer)),
        }
    }
}

impl<Ex, C> Clone for ChainExtractor<Ex, C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Ex, C> Deref for ChainExtractor<Ex, C> {
    type Target = ChainExtractorInner<Ex, C>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Ex, C> Extractor for ChainExtractor<Ex, C>
where
    Ex: Extractor + Send + Sync,
    C: Chainer<Ex::Extracted> + Send + Sync,
{
    type Extracted = C::Chained;

    type Error = ChainError<Ex::Error, C::Error>;

    
    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let extracted = self
            .extractor
            .extract(headers)
            .await
            .map_err(ChainError::Extract)?;

        let chained = self
            .chainer
            .chain(extracted)
            .await
            .map_err(ChainError::Chain)?;

        Ok(chained)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChainError<Ex, E> {
    #[error("Extraction error: {0}")]
    Extract(#[source] Ex),
    #[error("Chain error: {0}")]
    Chain(#[source] E),
}

#[cfg(feature = "axum")]
mod axum {
    use axum::response::{IntoResponse, Response};

    use super::ChainError;

    impl<Ex, E> IntoResponse for ChainError<Ex, E>
    where
        Ex: IntoResponse,
        E: IntoResponse,
    {
        fn into_response(self) -> Response {
            match self {
                ChainError::Extract(err) => err.into_response(),
                ChainError::Chain(err) => err.into_response(),
            }
        }
    }

    impl<Ex, E> From<ChainError<Ex, E>> for Response
    where
        Ex: IntoResponse,
        E: IntoResponse,
    {
        fn from(value: ChainError<Ex, E>) -> Self {
            value.into_response()
        }
    }
}
