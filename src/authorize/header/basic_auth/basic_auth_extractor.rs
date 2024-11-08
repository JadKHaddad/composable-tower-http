use http::HeaderMap;

pub trait BasicAuthExtractor {
    type Error;

    fn extract_basic_auth(&self, headers: &HeaderMap) -> Result<(String, String), Self::Error>;
}
