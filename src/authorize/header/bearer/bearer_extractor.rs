use http::HeaderMap;

pub trait BearerExtractor {
    type Error;

    fn extract_bearer<'a>(&self, headers: &'a HeaderMap) -> Result<&'a str, Self::Error>;
}

pub trait BearerExtractorExt: Sized + BearerExtractor {
    fn map_err<Fn>(self, map_err: Fn) -> ErrorMap<Self, Fn>;
}

impl<T> BearerExtractorExt for T
where
    T: Sized + BearerExtractor,
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

impl<B, Fn, E> BearerExtractor for ErrorMap<B, Fn>
where
    B: BearerExtractor + Sync,
    Fn: FnOnce(B::Error) -> E + Copy + Sync,
{
    type Error = E;

    #[tracing::instrument(skip_all)]
    fn extract_bearer<'a>(&self, headers: &'a HeaderMap) -> Result<&'a str, Self::Error> {
        self.inner.extract_bearer(headers).map_err(self.map_err)
    }
}
