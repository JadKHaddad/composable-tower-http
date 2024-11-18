use std::{collections::HashSet, ops::Deref, sync::Arc};

use crate::{
    authorize::header::basic_auth::basic_auth_extractor::BasicAuthExtractor, extract::Extractor,
};

use super::basic_auth_user::BasicAuthUser;

#[derive(Debug)]
pub struct DefaultBasicAuthAuthorizerInner<Ba> {
    basic_auth_extractor: Ba,
    users: HashSet<BasicAuthUser>,
}

impl<Ba> DefaultBasicAuthAuthorizerInner<Ba> {
    pub const fn new(basic_auth_extractor: Ba, users: HashSet<BasicAuthUser>) -> Self {
        Self {
            basic_auth_extractor,
            users,
        }
    }
}

#[derive(Debug)]
pub struct DefaultBasicAuthAuthorizer<Ba> {
    inner: Arc<DefaultBasicAuthAuthorizerInner<Ba>>,
}

impl<Ba> DefaultBasicAuthAuthorizer<Ba> {
    pub fn new(basic_auth_extractor: Ba, users: HashSet<BasicAuthUser>) -> Self {
        Self {
            inner: Arc::new(DefaultBasicAuthAuthorizerInner::new(
                basic_auth_extractor,
                users,
            )),
        }
    }
}

impl<Ba> Clone for DefaultBasicAuthAuthorizer<Ba> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Ba> Deref for DefaultBasicAuthAuthorizer<Ba> {
    type Target = DefaultBasicAuthAuthorizerInner<Ba>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Ba> Extractor for DefaultBasicAuthAuthorizer<Ba>
where
    Ba: BasicAuthExtractor + Send + Sync,
{
    type Extracted = BasicAuthUser;

    type Error = DefaultBasicAuthAuthorizeError<Ba::Error>;

    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let used_basic_auth = self
            .basic_auth_extractor
            .extract_basic_auth(headers)
            .map_err(DefaultBasicAuthAuthorizeError::BasicAuth)?;

        let basic_auth_user: BasicAuthUser = used_basic_auth.into();

        if self.users.contains(&basic_auth_user) {
            return Ok(basic_auth_user);
        }

        Err(DefaultBasicAuthAuthorizeError::Invalid)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DefaultBasicAuthAuthorizeError<Ba> {
    #[error("Basic auth extraction error: {0}")]
    BasicAuth(#[source] Ba),
    #[error("Invalid basic auth")]
    Invalid,
}

#[cfg(feature = "axum")]
mod axum {
    use axum::response::{IntoResponse, Response};
    use http::{header, StatusCode};

    use super::DefaultBasicAuthAuthorizeError;

    impl<Ba> IntoResponse for DefaultBasicAuthAuthorizeError<Ba>
    where
        Ba: std::error::Error,
    {
        fn into_response(self) -> Response {
            tracing::warn!(err = %self, "Unauthorized");

            (
                StatusCode::UNAUTHORIZED,
                [(
                    header::WWW_AUTHENTICATE,
                    "Basic realm=\"restricted\", charset=\"UTF-8\"",
                )],
            )
                .into_response()
        }
    }

    impl<Ba> From<DefaultBasicAuthAuthorizeError<Ba>> for Response
    where
        Ba: std::error::Error,
    {
        fn from(value: DefaultBasicAuthAuthorizeError<Ba>) -> Self {
            value.into_response()
        }
    }
}
