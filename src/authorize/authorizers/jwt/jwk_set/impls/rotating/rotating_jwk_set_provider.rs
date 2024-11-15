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
    pub async fn refresh_jwk_set<'a>(
        &'a self,
    ) -> Result<impl AsRef<JwkSet> + 'a, RotatingJwkSetProvideError<F::Error>> {
        tracing::debug!("Refreshing JWK set");

        let jwk_set = self
            .jwk_set_fetcher
            .fetch_jwk_set()
            .await
            .map_err(RotatingJwkSetProvideError::Fetch)?;

        *(self.holder.write().await) = JwkSetHolder {
            last_updated: Instant::now(),
            jwk_set,
        };

        let guard = self.holder.read().await;

        Ok(JwkSetReadGuard::new(guard))
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

        Ok(JwkSetReadGuard::new(guard))
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

impl AsRef<JwkSet> for JwkSetReadGuard<'_> {
    fn as_ref(&self) -> &JwkSet {
        self.0.as_ref()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RotatingJwkSetProvideError<F> {
    #[error("Failed to fetch JWK set: {0}")]
    Fetch(#[source] F),
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use jsonwebtoken::jwk::{
        AlgorithmParameters, CommonParameters, Jwk, JwkSet, OctetKeyParameters, OctetKeyType,
    };

    use crate::{
        authorize::authorizers::jwt::jwk_set::impls::rotating::jwk_set_fetcher::MockJwkSetFetcher,
        test::init_tracing,
    };

    use super::*;

    #[tokio::test]
    async fn jwk_set_will_rotate() {
        init_tracing();

        let mut jwk_set_fetcher = MockJwkSetFetcher::default();

        jwk_set_fetcher
            .expect_fetch_jwk_set()
            .times(1)
            .returning(|| Box::pin(async { Ok(JwkSet { keys: Vec::new() }) }));

        jwk_set_fetcher
            .expect_fetch_jwk_set()
            .times(1)
            .returning(|| {
                Box::pin(async {
                    Ok(JwkSet {
                        keys: vec![Jwk {
                            common: CommonParameters::default(),
                            algorithm: AlgorithmParameters::OctetKey(OctetKeyParameters {
                                key_type: OctetKeyType::Octet,
                                value: String::new(),
                            }),
                        }],
                    })
                })
            });

        let jwks_time_to_live_in_seconds = 1;

        let rotating_jwk_set_provider =
            RotatingJwkSetProvider::new(jwks_time_to_live_in_seconds, jwk_set_fetcher)
                .await
                .expect("Failed to create rotating jwk set provider");

        let on_creation_jwks = rotating_jwk_set_provider
            .provide_jwk_set()
            .await
            .expect("Failed to get jwk set")
            .as_ref()
            .clone();

        tokio::time::sleep(Duration::from_millis(2100)).await;

        let current_jwks = rotating_jwk_set_provider
            .provide_jwk_set()
            .await
            .expect("Failed to get jwk set")
            .as_ref()
            .clone();

        assert_ne!(on_creation_jwks, current_jwks)
    }
}
