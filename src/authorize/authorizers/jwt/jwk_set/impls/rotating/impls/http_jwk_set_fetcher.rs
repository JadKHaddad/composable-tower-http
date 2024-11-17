use jsonwebtoken::jwk::JwkSet;

use crate::authorize::authorizers::jwt::jwk_set::impls::rotating::jwk_set_fetcher::JwkSetFetcher;

#[derive(Debug)]
pub struct HttpJwkSetFetcher {
    jwks_uri: String,
    http_client: reqwest::Client,
}

impl HttpJwkSetFetcher {
    pub const fn new(jwks_uri: String, http_client: reqwest::Client) -> Self {
        Self {
            jwks_uri,
            http_client,
        }
    }

    pub async fn fetch(&self) -> Result<JwkSet, HttpJwkSetFetchError> {
        tracing::debug!("Fetching JWK set");

        let jwks = self
            .http_client
            .get(&self.jwks_uri)
            .send()
            .await
            .map_err(HttpJwkSetFetchError::Fetch)?
            .json::<JwkSet>()
            .await
            .map_err(HttpJwkSetFetchError::Parse)?;

        Ok(jwks)
    }
}

impl JwkSetFetcher for HttpJwkSetFetcher {
    type Error = HttpJwkSetFetchError;

    async fn fetch_jwk_set(&self) -> Result<JwkSet, Self::Error> {
        self.fetch().await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HttpJwkSetFetchError {
    #[error("Failed to fetch JWK set: {0}")]
    Fetch(#[source] reqwest::Error),
    #[error("Failed to parse JWK set: {0}")]
    Parse(#[source] reqwest::Error),
}
