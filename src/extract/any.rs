use super::extractor::Extractor;

#[derive(Debug, Clone)]
pub struct Any<L, F> {
    left: L,
    right: F,
}

impl<L, F> Any<L, F> {
    pub const fn new(left: L, right: F) -> Self {
        Self { left, right }
    }
}

impl<L, R> Extractor for Any<L, R>
where
    L: Extractor + Send + Sync,
    R: Extractor + Send + Sync,
    L::Extracted: From<R::Extracted>,
    L::Error: Send,
{
    type Extracted = L::Extracted;

    type Error = AnyError<L::Error, R::Error>;

    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        match self.left.extract(headers).await {
            Ok(extracted) => Ok(extracted),
            Err(left_error) => match self.right.extract(headers).await {
                Ok(extracted) => Ok(extracted.into()),
                Err(right_error) => Err(AnyError {
                    left: left_error,
                    right: right_error,
                }),
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Left: {left}, Right: {right}")]
pub struct AnyError<L, R> {
    pub left: L,
    pub right: R,
}

#[cfg(feature = "axum")]
mod axum {
    use axum::response::{IntoResponse, Response};

    use super::AnyError;

    impl<L, R> IntoResponse for AnyError<L, R>
    where
        L: IntoResponse,
    {
        fn into_response(self) -> Response {
            self.left.into_response()
        }
    }

    impl<L, R> From<AnyError<L, R>> for Response
    where
        L: IntoResponse,
    {
        fn from(value: AnyError<L, R>) -> Self {
            value.into_response()
        }
    }
}
