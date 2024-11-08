/// Extract authorized data produced by [`AuthorizationExtractor`](super::authorization_extractor::AuthorizationExtractor).
#[derive(Debug, Clone)]
pub struct Authorized<T>(pub T);

#[cfg(feature = "axum")]
mod axum {
    use axum::{async_trait, extract::FromRequestParts, http::request::Parts, Extension};
    use http::StatusCode;

    use super::{super::sealed_authorized::SealedAuthorized, Authorized};

    #[async_trait]
    impl<T, S> FromRequestParts<S> for Authorized<T>
    where
        T: Clone + Send + Sync + 'static,
        S: Send + Sync,
    {
        type Rejection = StatusCode;

        #[tracing::instrument(skip_all)]
        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
            let extracted =
                Extension::<SealedAuthorized<T>>::from_request_parts(parts, state).await;

            match extracted {
                Ok(Extension(SealedAuthorized(extracted))) => Ok(Authorized(extracted)),
                Err(_) => {
                    tracing::error!(
                        "Requested authorized extension was not found. Did you use `AuthorizationExtractor` with `ExtensionLayer`?"
                    );

                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
    }
}
