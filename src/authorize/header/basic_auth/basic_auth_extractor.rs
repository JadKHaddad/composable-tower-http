use http::HeaderMap;

pub trait BasicAuthExtractor {
    type Error;

    fn extract_basic_auth(&self, headers: &HeaderMap) -> Result<(String, String), Self::Error>;
}

pub trait BasicAuthExtractorExt: Sized + BasicAuthExtractor {
    fn map_err<Fn>(self, map_err: Fn) -> ErrorMap<Self, Fn>;
}

impl<T> BasicAuthExtractorExt for T
where
    T: Sized + BasicAuthExtractor,
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

impl<B, Fn, E> BasicAuthExtractor for ErrorMap<B, Fn>
where
    B: BasicAuthExtractor + Sync,
    Fn: FnOnce(B::Error) -> E + Copy + Sync,
{
    type Error = E;

    #[tracing::instrument(skip_all)]
    fn extract_basic_auth(&self, headers: &HeaderMap) -> Result<(String, String), Self::Error> {
        self.inner.extract_basic_auth(headers).map_err(self.map_err)
    }
}
