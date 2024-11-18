use http::HeaderMap;

use crate::error::InfallibleError;

use super::extractor::Extractor;

#[derive(Debug, Clone)]
pub struct Optional<T> {
    inner: T,
}

impl<T> Optional<T> {
    pub const fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<Ex> Extractor for Optional<Ex>
where
    Ex: Extractor + Sync,
{
    type Extracted = Option<Ex::Extracted>;

    type Error = InfallibleError;

    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        Ok(self.inner.extract(headers).await.ok())
    }
}
