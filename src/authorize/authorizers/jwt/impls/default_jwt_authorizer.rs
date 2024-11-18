use std::{marker::PhantomData, ops::Deref, sync::Arc};

use http::HeaderMap;
use jsonwebtoken::{decode, decode_header, errors::Error as JwtError, jwk::JwkSet, DecodingKey};
use serde::de::DeserializeOwned;

use crate::{
    authorize::{
        authorizers::jwt::jwk_set::jwk_set_provider::JwkSetProvider,
        header::bearer::BearerExtractor,
    },
    extract::Extractor,
};

use super::validation::Validation;

#[derive(Debug)]
pub struct DefaultJwtAuthorizerInner<Be, P, C> {
    bearer_extractor: Be,
    jwk_set_provider: P,
    validation: Validation,
    _claims: PhantomData<C>,
}

impl<Be, P, C> DefaultJwtAuthorizerInner<Be, P, C> {
    const fn new(bearer_extractor: Be, jwk_set_provider: P, validation: Validation) -> Self {
        Self {
            bearer_extractor,
            jwk_set_provider,
            validation,
            _claims: PhantomData,
        }
    }

    pub fn validate(&self, jwt: &str, jwks: &JwkSet) -> Result<C, DefaultJwtValidationError>
    where
        C: DeserializeOwned,
    {
        let header = decode_header(jwt).map_err(DefaultJwtValidationError::DecodeHeader)?;
        let kid = header.kid.ok_or(DefaultJwtValidationError::Kid)?;

        let jwk = jwks
            .find(&kid)
            .ok_or(DefaultJwtValidationError::MatchingJWK { kid })?;

        let decoding_key =
            DecodingKey::from_jwk(jwk).map_err(DefaultJwtValidationError::DecodingKey)?;

        let jsonwebtoen_validation = self.validation.to_jsonwebtoken_validation(header.alg);

        let token_data = decode::<C>(jwt, &decoding_key, &jsonwebtoen_validation)
            .map_err(DefaultJwtValidationError::DecodeData)?;

        Ok(token_data.claims)
    }
}

#[derive(Debug)]
pub struct DefaultJwtAuthorizer<Be, P, C> {
    inner: Arc<DefaultJwtAuthorizerInner<Be, P, C>>,
}

impl<Be, P, C> Clone for DefaultJwtAuthorizer<Be, P, C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Be, P, C> Deref for DefaultJwtAuthorizer<Be, P, C> {
    type Target = DefaultJwtAuthorizerInner<Be, P, C>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Be, P, C> DefaultJwtAuthorizer<Be, P, C> {
    pub fn new(bearer_extractor: Be, jwk_set_provider: P, validation: Validation) -> Self {
        Self {
            inner: Arc::new(DefaultJwtAuthorizerInner::new(
                bearer_extractor,
                jwk_set_provider,
                validation,
            )),
        }
    }
}

impl<Be, P, C> Extractor for DefaultJwtAuthorizer<Be, P, C>
where
    Be: BearerExtractor + Send + Sync,
    P: JwkSetProvider + Send + Sync,
    C: DeserializeOwned + Clone + Send + Sync + 'static,
{
    type Extracted = C;

    type Error = DefaultJwtAuthorizeError<Be::Error, P::Error>;

    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let bearer = self
            .bearer_extractor
            .extract_bearer(headers)
            .map_err(DefaultJwtAuthorizeError::Bearer)?;

        let jwks = self
            .jwk_set_provider
            .provide_jwk_set()
            .await
            .map_err(DefaultJwtAuthorizeError::JwkSet)?;

        self.validate(bearer, jwks.as_ref())
            .map_err(DefaultJwtAuthorizeError::Jwt)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DefaultJwtAuthorizeError<Be, P> {
    #[error("Bearer extraction error: {0}")]
    Bearer(#[source] Be),
    #[error("JWK set get error: {0}")]
    JwkSet(#[source] P),
    #[error("JWT validation error: {0}")]
    Jwt(
        #[source]
        #[from]
        DefaultJwtValidationError,
    ),
}

#[derive(Debug, thiserror::Error)]
pub enum DefaultJwtValidationError {
    #[error("Header decode error: {0}")]
    DecodeHeader(#[source] JwtError),
    #[error("Kid not found")]
    Kid,
    #[error("No matching JWK found for the given kid: {kid}")]
    MatchingJWK { kid: String },
    #[error("Decoding key creation error : {0}")]
    DecodingKey(#[source] JwtError),
    #[error("Data decode error: {0}")]
    DecodeData(#[source] JwtError),
}

/// A very interesting design choice.
///
/// # Usage
/// With this builder you are able to create a [`DefaultJwtAuthorizer`] like this
///
/// ```rust,ignore
/// let jwt_authorizer = DefaultJwtAuthorizerBuilder::new(
///     bearer_extractor,
///     jwk_set_provider,
///     validation,
/// )
/// .build::<Claims>();
/// ```
/// instead of this
/// ```rust,ignore
/// let jwt_authorizer =
///     DefaultJwtAuthorizer::<_, _, Claims>::new(bearer_extractor, jwk_set_provider, validation);
/// ```
#[derive(Debug)]
pub struct DefaultJwtAuthorizerBuilder<Be, P> {
    bearer_extractor: Be,
    jwk_set_provider: P,
    validation: Validation,
}

impl<Be, P> DefaultJwtAuthorizerBuilder<Be, P> {
    pub fn new(bearer_extractor: Be, jwk_set_provider: P, validation: Validation) -> Self {
        Self {
            bearer_extractor,
            jwk_set_provider,
            validation,
        }
    }

    pub fn build<C>(self) -> DefaultJwtAuthorizer<Be, P, C> {
        DefaultJwtAuthorizer::new(
            self.bearer_extractor,
            self.jwk_set_provider,
            self.validation,
        )
    }
}

#[cfg(feature = "axum")]
mod axum {
    use axum::response::{IntoResponse, Response};
    use http::{header, StatusCode};

    use super::DefaultJwtAuthorizeError;

    impl<Be, P> IntoResponse for DefaultJwtAuthorizeError<Be, P>
    where
        Be: std::error::Error,
        P: std::error::Error,
    {
        fn into_response(self) -> Response {
            tracing::warn!(err = %self, "Unauthorized");

            let status_code = match self {
                DefaultJwtAuthorizeError::Bearer(_) => StatusCode::UNAUTHORIZED,
                DefaultJwtAuthorizeError::JwkSet(_) => StatusCode::INTERNAL_SERVER_ERROR,
                DefaultJwtAuthorizeError::Jwt(_) => StatusCode::UNAUTHORIZED,
            };

            (status_code, [(header::WWW_AUTHENTICATE, "Bearer")]).into_response()
        }
    }

    impl<Be, P> From<DefaultJwtAuthorizeError<Be, P>> for Response
    where
        Be: std::error::Error,
        P: std::error::Error,
    {
        fn from(value: DefaultJwtAuthorizeError<Be, P>) -> Self {
            value.into_response()
        }
    }
}
