use http::HeaderMap;

pub trait HeaderExtractor {
    type Error;

    fn extract_header<'a>(&self, headers: &'a HeaderMap) -> Result<&'a str, Self::Error>;

    fn header_name(&self) -> &str;
}

pub trait HeaderExtractorExt: Sized + HeaderExtractor {
    fn map_err<Fn>(self, map_err: Fn) -> ErrorMap<Self, Fn>;
}

impl<T> HeaderExtractorExt for T
where
    T: Sized + HeaderExtractor,
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

impl<H, Fn, E> HeaderExtractor for ErrorMap<H, Fn>
where
    H: HeaderExtractor + Sync,
    Fn: FnOnce(H::Error) -> E + Copy + Sync,
{
    type Error = E;

    #[tracing::instrument(skip_all, fields(header_name = %self.inner.header_name()))]
    fn extract_header<'a>(&self, headers: &'a HeaderMap) -> Result<&'a str, Self::Error> {
        self.inner.extract_header(headers).map_err(self.map_err)
    }

    #[tracing::instrument(skip_all)]
    fn header_name(&self) -> &str {
        self.inner.header_name()
    }
}
