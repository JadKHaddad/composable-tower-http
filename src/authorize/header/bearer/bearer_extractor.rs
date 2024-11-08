use http::HeaderMap;

pub trait BearerExtractor {
    type Error;

    fn extract_bearer<'a>(&self, headers: &'a HeaderMap) -> Result<&'a str, Self::Error>;
}
