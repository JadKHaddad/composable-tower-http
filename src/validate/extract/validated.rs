#[derive(Debug)]
pub struct Validated<T>(pub T);

#[cfg(feature = "axum")]
mod axum {
    use axum::{async_trait, extract::FromRequestParts, http::request::Parts, Extension};
    use http::StatusCode;

    use super::{super::sealed_validated::SealedValidated, Validated};

    #[async_trait]
    impl<T, S> FromRequestParts<S> for Validated<T>
    where
        T: Clone + Send + Sync + 'static,
        S: Send + Sync,
    {
        type Rejection = StatusCode;

        #[tracing::instrument(skip_all)]
        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
            let extracted = Extension::<SealedValidated<T>>::from_request_parts(parts, state).await;

            match extracted {
                Ok(Extension(SealedValidated(extracted))) => Ok(Validated(extracted)),
                Err(_) => {
                    tracing::error!(
                        "Requested validated extension was not found. Did you use `Validator` with `ExtensionLayer`?"
                    );

                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
    }
}
