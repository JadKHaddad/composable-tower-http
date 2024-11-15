use std::{sync::Arc, time::Instant};

use jsonwebtoken::jwk::JwkSet;
use tokio::sync::{oneshot, RwLock, RwLockReadGuard};

use crate::authorize::authorizers::jwt::jwk_set::jwk_set_provider::JwkSetProvider;

use super::jwk_set_fetcher::JwkSetFetcher;

#[derive(Debug)]
pub struct JwkSetHolder<F>
where
    F: JwkSetFetcher,
    F::Error: std::error::Error + Clone + Send + Sync + 'static,
{
    last_updated: Instant,
    last_error: Option<BackgroundRotatingJwkSetProvideError<F::Error>>,
    jwk_set: JwkSet,
}

#[derive(Debug)]
pub struct BackgroundRotatingJwkSetProvider<F>
where
    F: JwkSetFetcher,
    F::Error: std::error::Error + Clone + Send + Sync + 'static,
{
    holder: Arc<RwLock<JwkSetHolder<F>>>,
    jwk_set_fetcher: Arc<F>,
    _cancellation_tx: oneshot::Sender<()>,
}

impl<F> BackgroundRotatingJwkSetProvider<F>
where
    F: JwkSetFetcher + Send + Sync + 'static,
    F::Error: std::error::Error + Clone + Send + Sync + 'static,
{
    pub async fn new(
        refresh_interval_in_seconds: u64,
        jwk_set_fetcher: F,
    ) -> Result<Self, BackgroundRotatingJwkSetProvideError<F::Error>> {
        let jwk_set = jwk_set_fetcher
            .fetch_jwk_set()
            .await
            .map_err(BackgroundRotatingJwkSetProvideError::Fetch)?;

        let last_updated = Instant::now();

        let holder = Arc::new(RwLock::new(JwkSetHolder {
            last_updated,
            last_error: None,
            jwk_set,
        }));

        let jwk_set_fetcher = Arc::new(jwk_set_fetcher);

        let (tx, rx) = oneshot::channel::<()>();

        tokio::spawn(Self::background_refresh_loop(
            refresh_interval_in_seconds,
            jwk_set_fetcher.clone(),
            holder.clone(),
            rx,
        ));

        Ok(Self {
            holder,
            jwk_set_fetcher,
            _cancellation_tx: tx,
        })
    }

    #[tracing::instrument(skip_all)]
    async fn background_refresh_loop(
        refresh_interval_in_seconds: u64,
        jwk_set_fetcher: Arc<F>,
        holder: Arc<RwLock<JwkSetHolder<F>>>,
        mut cancellation_rx: oneshot::Receiver<()>,
    ) {
        loop {
            tracing::debug!("Next refresh in {} seconds", refresh_interval_in_seconds);

            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(refresh_interval_in_seconds)) => {
                    if let Err(err) = Self::refresh_jwk_set_inner(&jwk_set_fetcher, &holder).await {
                        tracing::error!(?err, "Failed to refresh JWK set");
                    }
                }
                _ = &mut cancellation_rx => {
                    break;
                }
            }
        }

        tracing::debug!("Background refresh loop terminated");
    }

    #[tracing::instrument(name = "refresh_jwk_set", skip_all)]
    async fn refresh_jwk_set_inner<'a>(
        jwk_set_fetcher: &F,
        holder: &'a RwLock<JwkSetHolder<F>>,
    ) -> Result<impl AsRef<JwkSet> + 'a, BackgroundRotatingJwkSetProvideError<F::Error>> {
        tracing::debug!("Refreshing JWK set");

        let jwk_set = jwk_set_fetcher.fetch_jwk_set().await;

        let last_updated = Instant::now();

        match jwk_set {
            Ok(jwk_set) => {
                *(holder.write().await) = JwkSetHolder {
                    last_updated,
                    last_error: None,
                    jwk_set,
                };

                let guard = holder.read().await;

                Ok(JwkSetReadGuard::new(guard))
            }
            Err(err) => {
                let mut holder = holder.write().await;

                holder.last_updated = last_updated;
                holder.last_error = Some(BackgroundRotatingJwkSetProvideError::Fetch(err.clone()));

                Err(BackgroundRotatingJwkSetProvideError::Fetch(err))
            }
        }
    }

    pub async fn refresh_jwk_set(
        &self,
    ) -> Result<impl AsRef<JwkSet> + use<'_, F>, BackgroundRotatingJwkSetProvideError<F::Error>>
    {
        Self::refresh_jwk_set_inner(&self.jwk_set_fetcher, &self.holder).await
    }

    #[tracing::instrument(skip_all)]
    fn get(&self) -> &RwLock<JwkSetHolder<F>> {
        &self.holder
    }
}

impl<F> JwkSetProvider for BackgroundRotatingJwkSetProvider<F>
where
    F: JwkSetFetcher + Send + Sync + 'static,
    F::Error: std::error::Error + Clone + Send + Sync + 'static,
{
    type Error = BackgroundRotatingJwkSetProvideError<F::Error>;

    #[tracing::instrument(skip_all)]
    async fn provide_jwk_set(&self) -> Result<impl AsRef<JwkSet>, Self::Error> {
        let guard = self.get().read().await;

        if let Some(err) = &guard.last_error {
            return Err(err.clone());
        }

        Ok(JwkSetReadGuard::new(guard))
    }
}

impl<F> AsRef<JwkSet> for JwkSetHolder<F>
where
    F: JwkSetFetcher,
    F::Error: std::error::Error + Clone + Send + Sync + 'static,
{
    fn as_ref(&self) -> &JwkSet {
        &self.jwk_set
    }
}

#[derive(Debug)]
pub struct JwkSetReadGuard<'a, F>(RwLockReadGuard<'a, JwkSetHolder<F>>)
where
    F: JwkSetFetcher,
    F::Error: std::error::Error + Clone + Send + Sync + 'static;

impl<'a, F> JwkSetReadGuard<'a, F>
where
    F: JwkSetFetcher,
    F::Error: std::error::Error + Clone + Send + Sync + 'static,
{
    pub fn new(inner: RwLockReadGuard<'a, JwkSetHolder<F>>) -> Self {
        Self(inner)
    }
}

impl<F> AsRef<JwkSet> for JwkSetReadGuard<'_, F>
where
    F: JwkSetFetcher,
    F::Error: std::error::Error + Clone + Send + Sync + 'static,
{
    fn as_ref(&self) -> &JwkSet {
        self.0.as_ref()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum BackgroundRotatingJwkSetProvideError<F> {
    #[error("Failed to fetch JWK set: {0}")]
    Fetch(#[source] F),
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::bail;
    use jsonwebtoken::jwk::{
        AlgorithmParameters, CommonParameters, Jwk, JwkSet, OctetKeyParameters, OctetKeyType,
    };

    use crate::{
        authorize::authorizers::jwt::jwk_set::impls::rotating::jwk_set_fetcher::{
            JwkSetFetcherExt, MockJwkSetFetcher,
        },
        test::init_tracing,
    };

    use super::*;

    #[derive(Debug, Clone, thiserror::Error)]
    #[error("Failed to fetch JWK set")]
    struct MockJwkSetFetcherError;

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
            .times(2)
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

        let jwk_set_fetcher = jwk_set_fetcher.map_err(|_| MockJwkSetFetcherError);

        let jwks_refresh_interval_in_seconds = 1;

        let background_rotating_jwk_set_provider = BackgroundRotatingJwkSetProvider::new(
            jwks_refresh_interval_in_seconds,
            jwk_set_fetcher,
        )
        .await
        .expect("Failed to create background rotating jwk set provider");

        let on_creation_jwks = background_rotating_jwk_set_provider
            .provide_jwk_set()
            .await
            .expect("Failed to get jwk set")
            .as_ref()
            .clone();

        tokio::time::sleep(Duration::from_millis(2100)).await;

        let current_jwks = background_rotating_jwk_set_provider
            .provide_jwk_set()
            .await
            .expect("Failed to get jwk set")
            .as_ref()
            .clone();

        assert_ne!(on_creation_jwks, current_jwks);

        drop(background_rotating_jwk_set_provider);

        tokio::time::sleep(Duration::from_millis(2000)).await;
    }

    #[tokio::test]
    async fn error_in_background_will_return_error() {
        init_tracing();

        init_tracing();

        let mut jwk_set_fetcher = MockJwkSetFetcher::default();

        jwk_set_fetcher
            .expect_fetch_jwk_set()
            .times(1)
            .returning(|| Box::pin(async { Ok(JwkSet { keys: Vec::new() }) }));

        jwk_set_fetcher
            .expect_fetch_jwk_set()
            .times(2)
            .returning(|| Box::pin(async { bail!("Oh") }));

        let jwk_set_fetcher = jwk_set_fetcher.map_err(|_| MockJwkSetFetcherError);

        let jwks_refresh_interval_in_seconds = 1;

        let background_rotating_jwk_set_provider = BackgroundRotatingJwkSetProvider::new(
            jwks_refresh_interval_in_seconds,
            jwk_set_fetcher,
        )
        .await
        .expect("Failed to create background rotating jwk set provider");

        tokio::time::sleep(Duration::from_millis(2100)).await;

        let jwks_result = background_rotating_jwk_set_provider.provide_jwk_set().await;

        if jwks_result.is_ok() {
            panic!("Expected error")
        }
    }
}
