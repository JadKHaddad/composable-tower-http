use std::{ops::Deref, sync::Arc};

use crate::{extract::extractor::Extractor, validate::validator::Validator};

#[derive(Debug)]
pub struct ThenAuthorizerInner<A, T> {
    authorizer: A,
    then: T,
}

impl<A, T> ThenAuthorizerInner<A, T> {
    pub const fn new(authorizer: A, then: T) -> Self {
        Self { authorizer, then }
    }
}

#[derive(Debug)]
pub struct ThenAuthorizer<A, T> {
    inner: Arc<ThenAuthorizerInner<A, T>>,
}

impl<Ex, V> ThenAuthorizer<Ex, V> {
    pub fn new(extractor: Ex, validator: V) -> Self {
        Self {
            inner: Arc::new(ThenAuthorizerInner::new(extractor, validator)),
        }
    }
}

impl<Ex, V> Clone for ThenAuthorizer<Ex, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Ex, V> Deref for ThenAuthorizer<Ex, V> {
    type Target = ThenAuthorizerInner<Ex, V>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Ex, V> Extractor for ThenAuthorizer<Ex, V>
where
    Ex: Extractor + Send + Sync,
    V: Validator<Ex::Extracted> + Send + Sync,
{
    type Extracted = SealedValidated<Ex::Extracted>;

    type Error = ValidateError<Ex::Error, V::Error>;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let extracted = self
            .authorizer
            .extract(headers)
            .await
            .map_err(ValidateError::Extract)?;

        self.then
            .validate(&extracted)
            .await
            .map_err(ValidateError::Validate)?;

        Ok(SealedValidated(extracted))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidateError<Ex, E> {
    #[error("Extraction error: {0}")]
    Extract(#[source] Ex),
    #[error("Validation error: {0}")]
    Validate(#[source] E),
}

#[cfg(feature = "axum")]
mod axum {
    use axum::response::{IntoResponse, Response};

    use super::ValidateError;

    impl<Ex, E> IntoResponse for ValidateError<Ex, E>
    where
        Ex: std::error::Error + IntoResponse,
        E: std::error::Error + IntoResponse,
    {
        fn into_response(self) -> Response {
            tracing::warn!(err = %self, "Invalid");

            match self {
                ValidateError::Extract(err) => err.into_response(),
                ValidateError::Validate(err) => err.into_response(),
            }
        }
    }

    impl<Ex, E> From<ValidateError<Ex, E>> for Response
    where
        Ex: std::error::Error + IntoResponse,
        E: std::error::Error + IntoResponse,
    {
        fn from(value: ValidateError<Ex, E>) -> Self {
            value.into_response()
        }
    }
}
