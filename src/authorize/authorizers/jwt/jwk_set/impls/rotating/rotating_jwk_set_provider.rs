use std::time::Instant;

use jsonwebtoken::jwk::JwkSet;
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::authorize::authorizers::jwt::jwk_set::jwk_set_provider::JwkSetProvider;

use super::jwk_set_fetcher::JwkSetFetcher;

#[derive(Debug)]
pub struct JwkSetHolder {
    last_updated: Instant,
    jwk_set: JwkSet,
}

#[derive(Debug)]
pub struct RotatingJwkSetProvider<F> {
    time_to_live_in_seconds: u64,
    jwk_set_fetcher: F,
    holder: RwLock<JwkSetHolder>,
}

impl<F> RotatingJwkSetProvider<F>
where
    F: JwkSetFetcher,
{
    pub async fn new(
        time_to_live_in_seconds: u64,
        jwk_set_fetcher: F,
    ) -> Result<Self, RotatingJwkSetProvideError<F::Error>> {
        let jwk_set = jwk_set_fetcher
            .fetch_jwk_set()
            .await
            .map_err(RotatingJwkSetProvideError::Fetch)?;

        let last_updated = Instant::now();

        Ok(Self {
            time_to_live_in_seconds,
            jwk_set_fetcher,
            holder: RwLock::new(JwkSetHolder {
                last_updated,
                jwk_set,
            }),
        })
    }

    #[tracing::instrument(skip_all)]
    async fn refresh_jwk_set(&self) -> Result<(), RotatingJwkSetProvideError<F::Error>> {
        tracing::debug!("Refreshing jwks");

        let jwk_set = self
            .jwk_set_fetcher
            .fetch_jwk_set()
            .await
            .map_err(RotatingJwkSetProvideError::Fetch)?;

        let mut inner = self.holder.write().await;

        inner.jwk_set = jwk_set;
        inner.last_updated = Instant::now();

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn get(&self) -> Result<&RwLock<JwkSetHolder>, RotatingJwkSetProvideError<F::Error>> {
        let last_updated = self.holder.read().await.last_updated;

        if last_updated.elapsed().as_secs() > self.time_to_live_in_seconds {
            self.refresh_jwk_set().await?;
        }

        Ok(&self.holder)
    }
}

impl<F> JwkSetProvider for RotatingJwkSetProvider<F>
where
    F: JwkSetFetcher + Sync,
    F::Error: Send,
{
    type Error = RotatingJwkSetProvideError<F::Error>;

    #[tracing::instrument(skip_all)]
    async fn provide_jwk_set(&self) -> Result<impl AsRef<JwkSet>, Self::Error> {
        let guard = self.get().await?.read().await;
        let guard = JwkSetReadGuard::new(guard);

        Ok(guard)
    }
}

impl AsRef<JwkSet> for JwkSetHolder {
    fn as_ref(&self) -> &JwkSet {
        &self.jwk_set
    }
}

#[derive(Debug)]
pub struct JwkSetReadGuard<'a>(RwLockReadGuard<'a, JwkSetHolder>);

impl<'a> JwkSetReadGuard<'a> {
    pub fn new(inner: RwLockReadGuard<'a, JwkSetHolder>) -> Self {
        Self(inner)
    }
}

impl<'a> AsRef<JwkSet> for JwkSetReadGuard<'a> {
    fn as_ref(&self) -> &JwkSet {
        self.0.as_ref()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RotatingJwkSetProvideError<F> {
    #[error("Failed to fetch JWK set: {0}")]
    Fetch(#[source] F),
}
