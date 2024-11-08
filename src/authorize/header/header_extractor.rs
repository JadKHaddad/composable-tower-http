use http::HeaderMap;

pub trait HeaderExtractor {
    type Error;

    fn extract_header<'a>(&self, headers: &'a HeaderMap) -> Result<&'a str, Self::Error>;
}
