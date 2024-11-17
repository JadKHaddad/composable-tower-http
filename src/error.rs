use std::convert::Infallible;

#[derive(Debug, Clone, thiserror::Error)]
#[error("Infallible")]
#[repr(transparent)]
pub struct InfallibleError(Infallible);

#[cfg(feature = "axum")]
mod axum {
    use axum::response::{IntoResponse, Response};

    use super::InfallibleError;

    impl IntoResponse for InfallibleError {
        fn into_response(self) -> Response {
            ().into_response()
        }
    }

    impl From<InfallibleError> for Response {
        fn from(value: InfallibleError) -> Self {
            value.into_response()
        }
    }
}
