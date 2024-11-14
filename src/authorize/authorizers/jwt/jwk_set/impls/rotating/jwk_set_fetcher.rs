use std::future::Future;

use jsonwebtoken::jwk::JwkSet;

#[cfg_attr(test, mockall::automock(type Error=();))]
pub trait JwkSetFetcher {
    type Error;

    fn fetch_jwk_set(&self) -> impl Future<Output = Result<JwkSet, Self::Error>> + Send;
}
