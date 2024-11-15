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
    fn map<Fn>(self, map: Fn) -> Map<Self, Fn>;

    fn async_map<Fn>(self, map: Fn) -> AsyncMap<Self, Fn>;

    fn map_err<Fn>(self, map_err: Fn) -> ErrorMap<Self, Fn>;

    fn convert<Fn>(self, convert: Fn) -> Convert<Self, Fn>;

    fn async_convert<Fn>(self, convert: Fn) -> AsyncConvert<Self, Fn>;

    fn extracted(self) -> AuthorizationExtractor<Self>;
}

impl<T> AuthorizerExt for T
where
    T: Sized + Authorizer,
{
    fn map<Fn>(self, map: Fn) -> Map<Self, Fn> {
        Map::new(self, map)
    }

    fn async_map<Fn>(self, map: Fn) -> AsyncMap<Self, Fn> {
        AsyncMap::new(self, map)
    }

    fn map_err<Fn>(self, map_err: Fn) -> ErrorMap<Self, Fn> {
        ErrorMap::new(self, map_err)
    }

    fn convert<Fn>(self, convert: Fn) -> Convert<Self, Fn> {
        Convert::new(self, convert)
    }

    fn async_convert<Fn>(self, convert: Fn) -> AsyncConvert<Self, Fn> {
        AsyncConvert::new(self, convert)
    }

    fn extracted(self) -> AuthorizationExtractor<T> {
        AuthorizationExtractor::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct Map<T, Fn> {
    inner: T,
    map: Fn,
}

impl<T, Fn> Map<T, Fn> {
    pub const fn new(inner: T, map: Fn) -> Self {
        Self { inner, map }
    }
}

#[derive(Debug, Clone)]
pub struct AsyncMap<T, Fn> {
    inner: T,
    map: Fn,
}

impl<T, Fn> AsyncMap<T, Fn> {
    pub const fn new(inner: T, map: Fn) -> Self {
        Self { inner, map }
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

#[derive(Debug, Clone)]
pub struct Convert<T, Fn> {
    inner: T,
    convert: Fn,
}

impl<T, Fn> Convert<T, Fn> {
    pub const fn new(inner: T, convert: Fn) -> Self {
        Self { inner, convert }
    }
}

#[derive(Debug, Clone)]
pub struct AsyncConvert<T, Fn> {
    inner: T,
    convert: Fn,
}

impl<T, Fn> AsyncConvert<T, Fn> {
    pub const fn new(inner: T, convert: Fn) -> Self {
        Self { inner, convert }
    }
}

impl<A, Fn, T> Authorizer for Map<A, Fn>
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

impl<A, Fn, T, Fut> Authorizer for AsyncMap<A, Fn>
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

impl<A, Fn, E> Authorizer for ErrorMap<A, Fn>
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

impl<A, Fn, T, E> Authorizer for Convert<A, Fn>
where
    A: Authorizer + Sync,
    Fn: FnOnce(Result<A::Authorized, A::Error>) -> Result<T, E> + Copy + Sync,
    T: Clone + Send + Sync + 'static,
{
    type Authorized = T;

    type Error = E;

    #[tracing::instrument(skip_all)]
    async fn authorize(&self, headers: &HeaderMap) -> Result<Self::Authorized, Self::Error> {
        let authorized = self.inner.authorize(headers).await;

        (self.convert)(authorized)
    }
}

impl<A, Fn, T, E, Fut> Authorizer for AsyncConvert<A, Fn>
where
    A: Authorizer + Sync,
    Fn: FnOnce(Result<A::Authorized, A::Error>) -> Fut + Copy + Sync,
    Fut: Future<Output = Result<T, E>> + Send,
    T: Clone + Send + Sync + 'static,
{
    type Authorized = T;

    type Error = E;

    #[tracing::instrument(skip_all)]
    async fn authorize(&self, headers: &HeaderMap) -> Result<Self::Authorized, Self::Error> {
        let authorized = self.inner.authorize(headers).await;

        (self.convert)(authorized).await
    }
}
