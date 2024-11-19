use std::future::Future;

use jsonwebtoken::jwk::JwkSet;

#[cfg_attr(test, mockall::automock(type Error=anyhow::Error;))]
pub trait JwkSetFetcher {
    type Error;

    fn fetch_jwk_set(&self) -> impl Future<Output = Result<JwkSet, Self::Error>> + Send;
}

pub trait JwkSetFetcherExt: Sized + JwkSetFetcher {
    fn map_err<Fn>(self, map_err: Fn) -> MapError<Self, Fn>;
}

impl<T> JwkSetFetcherExt for T
where
    T: Sized + JwkSetFetcher,
{
    fn map_err<Fn>(self, map_err: Fn) -> MapError<Self, Fn> {
        MapError::new(self, map_err)
    }
}

#[derive(Debug, Clone)]
pub struct MapError<T, Fn> {
    inner: T,
    map_err: Fn,
}

impl<T, Fn> MapError<T, Fn> {
    pub const fn new(inner: T, map_err: Fn) -> Self {
        Self { inner, map_err }
    }
}

impl<J, Fn, E> JwkSetFetcher for MapError<J, Fn>
where
    J: JwkSetFetcher + Sync,
    Fn: FnOnce(J::Error) -> E + Clone + Sync,
{
    type Error = E;

    async fn fetch_jwk_set(&self) -> Result<JwkSet, Self::Error> {
        self.inner
            .fetch_jwk_set()
            .await
            .map_err(|err| (self.map_err.clone())(err))
    }
}
