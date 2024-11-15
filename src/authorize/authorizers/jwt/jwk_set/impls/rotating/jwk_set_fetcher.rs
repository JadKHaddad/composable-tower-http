use std::future::Future;

use jsonwebtoken::jwk::JwkSet;

#[cfg_attr(test, mockall::automock(type Error=anyhow::Error;))]
pub trait JwkSetFetcher {
    type Error;

    fn fetch_jwk_set(&self) -> impl Future<Output = Result<JwkSet, Self::Error>> + Send;
}

pub trait JwkSetFetcherExt: Sized + JwkSetFetcher {
    fn map_err<Fn>(self, map_err: Fn) -> ErrorMap<Self, Fn>;
}

impl<T> JwkSetFetcherExt for T
where
    T: Sized + JwkSetFetcher,
{
    fn map_err<Fn>(self, map_err: Fn) -> ErrorMap<Self, Fn> {
        ErrorMap::new(self, map_err)
    }
}

#[derive(Debug, Clone)]
pub struct ErrorMap<T, Fn> {
    inner: T,
    map_err: Fn,
}

impl<T, Fn> ErrorMap<T, Fn> {
    pub const fn new(inner: T, map_err: Fn) -> Self {
        Self { inner, map_err }
    }
}

impl<J, Fn, E> JwkSetFetcher for ErrorMap<J, Fn>
where
    J: JwkSetFetcher + Sync,
    Fn: FnOnce(J::Error) -> E + Copy + Sync,
{
    type Error = E;

    #[tracing::instrument(skip_all)]
    async fn fetch_jwk_set(&self) -> Result<JwkSet, Self::Error> {
        self.inner
            .fetch_jwk_set()
            .await
            .map_err(|err| (self.map_err)(err))
    }
}
