use std::future::Future;

use http::HeaderMap;

pub trait Authorizer {
    type Authorized: Clone + Send + Sync + 'static;

    type Error;

    fn authorize(
        &self,
        headers: &HeaderMap,
    ) -> impl Future<Output = Result<Self::Authorized, Self::Error>> + Send;
}
