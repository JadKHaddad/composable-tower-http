use std::{ops::Deref, sync::Arc};

use crate::extract::extractor::Extractor;

use super::Chain;

#[derive(Debug)]
pub struct ChainExtractorInner<Ex, C> {
    extractor: Ex,
    chain: C,
}

impl<Ex, C> ChainExtractorInner<Ex, C> {
    pub const fn new(extractor: Ex, chain: C) -> Self {
        Self { extractor, chain }
    }
}

#[derive(Debug)]
pub struct ChainExtractor<Ex, C> {
    inner: Arc<ChainExtractorInner<Ex, C>>,
}

impl<Ex, C> ChainExtractor<Ex, C> {
    pub fn new(extractor: Ex, chain: C) -> Self {
        Self {
            inner: Arc::new(ChainExtractorInner::new(extractor, chain)),
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
    C: Chain<Ex::Extracted> + Send + Sync,
{
    type Extracted = C::Extracted;

    type Error = ChainError<Ex::Error, C::Error>;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let extracted = self
            .extractor
            .extract(headers)
            .await
            .map_err(ChainError::Extract)?;

        let extracted = self
            .chain
            .chain(extracted)
            .await
            .map_err(ChainError::Chain)?;

        Ok(extracted)
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
        Ex: std::error::Error + IntoResponse,
        E: std::error::Error + IntoResponse,
    {
        fn into_response(self) -> Response {
            tracing::warn!(err = %self, "Invalid");

            match self {
                ChainError::Extract(err) => err.into_response(),
                ChainError::Chain(err) => err.into_response(),
            }
        }
    }

    impl<Ex, E> From<ChainError<Ex, E>> for Response
    where
        Ex: std::error::Error + IntoResponse,
        E: std::error::Error + IntoResponse,
    {
        fn from(value: ChainError<Ex, E>) -> Self {
            value.into_response()
        }
    }
}