#[derive(Debug, Clone)]
pub struct Extracted<T>(pub T);

#[cfg(feature = "axum")]
mod axum {
    use axum::{async_trait, extract::FromRequestParts, http::request::Parts, Extension};
    use http::StatusCode;

    use crate::extract::sealed_extracted::SealedExtracted;

    use super::Extracted;

    #[async_trait]
    impl<T, S> FromRequestParts<S> for Extracted<T>
    where
        T: Clone + Send + Sync + 'static,
        S: Send + Sync,
    {
        type Rejection = StatusCode;

        
        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
            let extracted = Extension::<SealedExtracted<T>>::from_request_parts(parts, state).await;

            match extracted {
                Ok(Extension(SealedExtracted(extracted))) => Ok(Extracted(extracted)),
                Err(_) => {
                    tracing::error!(
                        "Requested extracted extension was not found. Did you use `Extractor` with `ExtensionLayer`?"
                    );

                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
    }
}
