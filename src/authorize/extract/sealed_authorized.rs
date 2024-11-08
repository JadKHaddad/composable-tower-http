use std::ops::Deref;

/// This struct can only be created by [`AuthorizationExtractor`](super::authorization_extractor::AuthorizationExtractor).
///
/// Use [`Authorized`](super::authorized::Authorized) to extract the data produced by [`AuthorizationExtractor`](super::authorization_extractor::AuthorizationExtractor).
///
/// Because [`Authorized`](super::authorized::Authorized) extracts only [`SealedAuthorized`] data and [`SealedAuthorized`] can only be created by [`AuthorizationExtractor`](super::authorization_extractor::AuthorizationExtractor),
/// it is safe to assume that the data has been processed by [`AuthorizationExtractor`](super::authorization_extractor::AuthorizationExtractor) and not been created by arbitrary code.
#[derive(Debug, Clone)]
pub struct SealedAuthorized<T>(pub(super) T);

impl<T> SealedAuthorized<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for SealedAuthorized<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
