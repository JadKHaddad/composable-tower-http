use http::HeaderMap;

use crate::{authorize::authorizer::Authorizer, extract::extractor::Extractor};

use super::sealed_authorized::SealedAuthorized;

#[derive(Debug, Clone)]
pub struct AuthorizationExtractor<A> {
    authorizer: A,
}

impl<A> AuthorizationExtractor<A> {
    pub const fn new(authorizer: A) -> Self {
        Self { authorizer }
    }
}

impl<A> Extractor for AuthorizationExtractor<A>
where
    A: Authorizer + Sync,
{
    type Extracted = SealedAuthorized<A::Authorized>;

    type Error = A::Error;

    #[tracing::instrument(skip_all)]
    async fn extract(&self, headers: &HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let authorized = self.authorizer.authorize(headers).await?;

        Ok(SealedAuthorized(authorized))
    }
}
