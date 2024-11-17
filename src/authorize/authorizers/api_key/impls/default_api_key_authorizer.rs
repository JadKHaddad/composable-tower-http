use std::{borrow::Cow, collections::HashSet, ops::Deref, sync::Arc};

use crate::{authorize::header::header_extractor::HeaderExtractor, extract::extractor::Extractor};

use super::api_key::ApiKey;

#[derive(Debug)]
pub struct DefaultApiKeyAuthorizerInner<H> {
    header_extractor: H,
    valid_api_keys: HashSet<ApiKey>,
}

impl<H> DefaultApiKeyAuthorizerInner<H> {
    pub const fn new(header_extractor: H, valid_api_keys: HashSet<ApiKey>) -> Self {
        Self {
            header_extractor,
            valid_api_keys,
        }
    }
}

#[derive(Debug)]
pub struct DefaultApiKeyAuthorizer<H> {
    inner: Arc<DefaultApiKeyAuthorizerInner<H>>,
}

impl<H> DefaultApiKeyAuthorizer<H> {
    pub fn new(header_extractor: H, valid_api_keys: HashSet<ApiKey>) -> Self {
        Self {
            inner: Arc::new(DefaultApiKeyAuthorizerInner::new(
                header_extractor,
                valid_api_keys,
            )),
        }
    }
}

impl<H> Clone for DefaultApiKeyAuthorizer<H> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<H> Deref for DefaultApiKeyAuthorizer<H> {
    type Target = DefaultApiKeyAuthorizerInner<H>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<H> Extractor for DefaultApiKeyAuthorizer<H>
where
    H: HeaderExtractor + Send + Sync,
{
    type Extracted = ApiKey;

    type Error = DefaultApiKeyAuthorizeError<H::Error>;

    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let api_key_value = self
            .header_extractor
            .extract_header(headers)
            .map_err(DefaultApiKeyAuthorizeError::Header)?;

        let used_api_key = ApiKey::new(Cow::from(api_key_value.to_string()));

        if self.valid_api_keys.contains(&used_api_key) {
            return Ok(used_api_key);
        }

        Err(DefaultApiKeyAuthorizeError::Invalid)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DefaultApiKeyAuthorizeError<H> {
    #[error("Header extraction error: {0}")]
    Header(#[source] H),
    #[error("Invalid API key")]
    Invalid,
}

#[cfg(feature = "axum")]
mod axum {
    use axum::response::{IntoResponse, Response};
    use http::StatusCode;

    use super::DefaultApiKeyAuthorizeError;

    impl<H> IntoResponse for DefaultApiKeyAuthorizeError<H>
    where
        H: std::error::Error,
    {
        fn into_response(self) -> Response {
            tracing::warn!(err = %self, "Unauthorized");

            StatusCode::UNAUTHORIZED.into_response()
        }
    }

    impl<H> From<DefaultApiKeyAuthorizeError<H>> for Response
    where
        H: std::error::Error,
    {
        fn from(value: DefaultApiKeyAuthorizeError<H>) -> Self {
            value.into_response()
        }
    }
}
