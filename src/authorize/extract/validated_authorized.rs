#[derive(Debug, Clone)]
pub struct ValidatedAuthorized<T>(pub T);

#[cfg(feature = "axum")]
mod axum {
    use axum::{async_trait, extract::FromRequestParts, http::request::Parts, Extension};
    use http::StatusCode;

    use crate::validate::extract::sealed_validated::SealedValidated;

    use super::{super::sealed_authorized::SealedAuthorized, ValidatedAuthorized};

    #[async_trait]
    impl<T, S> FromRequestParts<S> for ValidatedAuthorized<T>
    where
        T: Clone + Send + Sync + 'static,
        S: Send + Sync,
    {
        type Rejection = StatusCode;

        #[tracing::instrument(skip_all)]
        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
            let extracted =
                Extension::<SealedValidated<SealedAuthorized<T>>>::from_request_parts(parts, state)
                    .await;

            match extracted {
                Ok(Extension(SealedValidated(SealedAuthorized(extracted)))) => {
                    Ok(ValidatedAuthorized(extracted))
                }
                Err(_) => {
                    tracing::error!(
                    "Requested validated authorized extension was not found. Did you use `AuthorizationExtractor` with `Validator` and `ExtensionLayer`?"
                );

                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
    }
}
