use crate::authorize::authorizer::Authorizer;

use super::authorization_extractor::AuthorizationExtractor;

pub trait AuthorizedExt: Sized + Authorizer {
    fn authorized(self) -> AuthorizationExtractor<Self>;
}

impl<T> AuthorizedExt for T
where
    T: Sized + Authorizer,
{
    fn authorized(self) -> AuthorizationExtractor<T> {
        AuthorizationExtractor::new(self)
    }
}
