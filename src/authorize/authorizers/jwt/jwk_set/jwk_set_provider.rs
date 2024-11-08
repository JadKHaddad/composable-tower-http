use std::future::Future;

use jsonwebtoken::jwk::JwkSet;

pub trait JwkSetProvider {
    type Error;

    fn provide_jwk_set(
        &self,
    ) -> impl Future<Output = Result<impl AsRef<JwkSet>, Self::Error>> + Send;
}
