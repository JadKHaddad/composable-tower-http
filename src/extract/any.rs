use super::extractor::Extractor;

#[derive(Debug)]
pub struct Any<Ex, const IS_ROOT: bool> {
    inner: Ex,
}

impl<Ex, const IS_ROOT: bool> Any<Ex, IS_ROOT> {
    pub const fn new(inner: Ex) -> Any<Ex, true> {
        Any { inner }
    }
}

impl<Ex> Extractor for Any<Ex, true>
where
    Ex: Extractor + Sync,
{
    type Extracted = Ex::Extracted;

    type Error = Ex::Error;

    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        self.inner.extract(headers).await
    }
}
