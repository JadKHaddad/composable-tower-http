use std::future::Future;

use jsonwebtoken::jwk::JwkSet;

pub trait JwkSetFetcher {
    type Error;

    fn fetch_jwk_set(&self) -> impl Future<Output = Result<JwkSet, Self::Error>> + Send;
}
