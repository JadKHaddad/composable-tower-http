use std::future::Future;

use http::HeaderMap;

use super::extract::authorization_extractor::AuthorizationExtractor;

pub trait Authorizer {
    type Authorized: Clone + Send + Sync + 'static;

    type Error;

    fn authorize(
        &self,
        headers: &HeaderMap,
    ) -> impl Future<Output = Result<Self::Authorized, Self::Error>> + Send;
}

pub trait AuthorizerExt: Sized + Authorizer {
    fn map<Fn>(self, map: Fn) -> Mapper<Self, Fn>;

    fn async_map<Fn>(self, map: Fn) -> AsyncMapper<Self, Fn>;

    fn map_err<Fn>(self, map_err: Fn) -> ErrorMapper<Self, Fn>;

    fn extracted(self) -> AuthorizationExtractor<Self>;
}

impl<T> AuthorizerExt for T
where
    T: Sized + Authorizer,
{
    fn map<Fn>(self, map: Fn) -> Mapper<Self, Fn> {
        Mapper::new(self, map)
    }

    fn async_map<Fn>(self, map: Fn) -> AsyncMapper<Self, Fn> {
        AsyncMapper::new(self, map)
    }

    fn map_err<Fn>(self, map_err: Fn) -> ErrorMapper<Self, Fn> {
        ErrorMapper::new(self, map_err)
    }

    fn extracted(self) -> AuthorizationExtractor<T> {
        AuthorizationExtractor::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct Mapper<T, Fn> {
    inner: T,
    map: Fn,
}

impl<T, Fn> Mapper<T, Fn> {
    pub const fn new(inner: T, map: Fn) -> Self {
        Self { inner, map }
    }
}

#[derive(Debug, Clone)]
pub struct AsyncMapper<T, Fn> {
    inner: T,
    map: Fn,
}

impl<T, Fn> AsyncMapper<T, Fn> {
    pub const fn new(inner: T, map: Fn) -> Self {
        Self { inner, map }
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

impl<A, Fn, T> Authorizer for Mapper<A, Fn>
where
    A: Authorizer + Sync,
    Fn: FnOnce(A::Authorized) -> T + Copy + Sync,
    T: Clone + Send + Sync + 'static,
{
    type Authorized = T;

    type Error = A::Error;

    #[tracing::instrument(skip_all)]
    async fn authorize(&self, headers: &HeaderMap) -> Result<Self::Authorized, Self::Error> {
        self.inner
            .authorize(headers)
            .await
            .map(|auth| (self.map)(auth))
    }
}

impl<A, Fn, T, Fut> Authorizer for AsyncMapper<A, Fn>
where
    A: Authorizer + Sync,
    Fn: FnOnce(A::Authorized) -> Fut + Copy + Sync,
    Fut: Future<Output = T> + Send,
    T: Clone + Send + Sync + 'static,
{
    type Authorized = T;

    type Error = A::Error;

    #[tracing::instrument(skip_all)]
    async fn authorize(&self, headers: &HeaderMap) -> Result<Self::Authorized, Self::Error> {
        let authorized = self.inner.authorize(headers).await?;

        let mapped = (self.map)(authorized).await;

        Ok(mapped)
    }
}

impl<A, Fn, E> Authorizer for ErrorMapper<A, Fn>
where
    A: Authorizer + Sync,
    Fn: FnOnce(A::Error) -> E + Copy + Sync,
{
    type Authorized = A::Authorized;

    type Error = E;

    #[tracing::instrument(skip_all)]
    async fn authorize(&self, headers: &HeaderMap) -> Result<Self::Authorized, Self::Error> {
        self.inner
            .authorize(headers)
            .await
            .map_err(|err| (self.map_err)(err))
    }
}
