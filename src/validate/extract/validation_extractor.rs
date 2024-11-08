use std::{ops::Deref, sync::Arc};

use crate::{extract::extractor::Extractor, validate::validator::Validator};

use super::sealed_validated::SealedValidated;

#[derive(Debug)]
pub struct ValidationExtractorInner<Ex, V> {
    extractor: Ex,
    validator: V,
}

impl<Ex, V> ValidationExtractorInner<Ex, V> {
    pub const fn new(extractor: Ex, validator: V) -> Self {
        Self {
            extractor,
            validator,
        }
    }
}

#[derive(Debug)]
pub struct ValidationExtractor<Ex, V> {
    inner: Arc<ValidationExtractorInner<Ex, V>>,
}

impl<Ex, V> ValidationExtractor<Ex, V> {
    pub fn new(extractor: Ex, validator: V) -> Self {
        Self {
            inner: Arc::new(ValidationExtractorInner::new(extractor, validator)),
        }
    }
}

impl<Ex, V> Clone for ValidationExtractor<Ex, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Ex, V> Deref for ValidationExtractor<Ex, V> {
    type Target = ValidationExtractorInner<Ex, V>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Ex, V> Extractor for ValidationExtractor<Ex, V>
where
    Ex: Extractor + Send + Sync,
    V: Validator<Ex::Extracted> + Send + Sync,
{
    type Extracted = SealedValidated<Ex::Extracted>;

    type Error = ValidateError<Ex::Error, V::Error>;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let extracted = self
            .extractor
            .extract(headers)
            .await
            .map_err(ValidateError::Authorize)?;

        self.validator
            .validate(&extracted)
            .await
            .map_err(ValidateError::Validate)?;

        Ok(SealedValidated(extracted))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidateError<Ex, E> {
    #[error("Authorization error: {0}")]
    Authorize(#[source] Ex),
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
                ValidateError::Authorize(err) => err.into_response(),
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
