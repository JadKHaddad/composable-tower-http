use std::future::Future;

use jsonwebtoken::jwk::JwkSet;

pub trait JwkSetProvider {
    type Error;

    fn provide_jwk_set(
        &self,
    ) -> impl Future<Output = Result<impl AsRef<JwkSet>, Self::Error>> + Send;
}

pub trait JwkSetProviderExt: Sized + JwkSetProvider {
    fn map_err<Fn>(self, map_err: Fn) -> ErrorMapper<Self, Fn>;
}

impl<T> JwkSetProviderExt for T
where
    T: Sized + JwkSetProvider,
{
    fn map_err<Fn>(self, map_err: Fn) -> ErrorMapper<Self, Fn> {
        ErrorMapper::new(self, map_err)
    }
}

#[derive(Debug, Clone)]
pub struct ErrorMapper<T, Fn> {
    inner: T,
    map_err: Fn,
}

impl<T, Fn> ErrorMapper<T, Fn> {
    pub const fn new(inner: T, map_err: Fn) -> Self {
        Self { inner, map_err }
    }
}

impl<J, Fn, E> JwkSetProvider for ErrorMapper<J, Fn>
where
    J: JwkSetProvider + Sync,
    Fn: FnOnce(J::Error) -> E + Copy + Sync,
{
    type Error = E;

    #[tracing::instrument(skip_all)]
    async fn provide_jwk_set(&self) -> Result<impl AsRef<JwkSet>, Self::Error> {
        self.inner
            .provide_jwk_set()
            .await
            .map_err(|err| (self.map_err)(err))
    }
}
