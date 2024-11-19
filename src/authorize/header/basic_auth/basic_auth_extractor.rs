use http::HeaderMap;

pub trait BasicAuthExtractor {
    type Error;

    fn extract_basic_auth(&self, headers: &HeaderMap) -> Result<(String, String), Self::Error>;
}

pub trait BasicAuthExtractorExt: Sized + BasicAuthExtractor {
    fn map_err<Fn>(self, map_err: Fn) -> MapError<Self, Fn>;
}

impl<T> BasicAuthExtractorExt for T
where
    T: Sized + BasicAuthExtractor,
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

impl<B, Fn, E> BasicAuthExtractor for MapError<B, Fn>
where
    B: BasicAuthExtractor + Sync,
    Fn: FnOnce(B::Error) -> E + Clone + Sync,
{
    type Error = E;

    fn extract_basic_auth(&self, headers: &HeaderMap) -> Result<(String, String), Self::Error> {
        self.inner
            .extract_basic_auth(headers)
            .map_err(self.map_err.clone())
    }
}
