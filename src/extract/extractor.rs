use std::future::Future;

use http::HeaderMap;

pub trait Extractor {
    type Extracted: Clone + Send + Sync;

    type Error;

    fn extract(
        &self,
        headers: &HeaderMap,
    ) -> impl Future<Output = Result<Self::Extracted, Self::Error>> + Send;
}
