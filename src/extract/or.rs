use super::extractor::Extractor;

#[derive(Debug, Clone)]
pub struct OrExtractor<L, R> {
    left: L,
    right: R,
}

impl<L, R> OrExtractor<L, R> {
    pub const fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}

#[derive(Debug, Clone)]
pub enum Or<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Extractor for OrExtractor<L, R>
where
    L: Extractor + Send + Sync,
    R: Extractor + Send + Sync,
    L::Error: Send,
{
    type Extracted = Or<L::Extracted, R::Extracted>;

    type Error = OrError<L::Error, R::Error>;

    
    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        match self.left.extract(headers).await {
            Ok(extracted) => Ok(Or::Left(extracted)),
            Err(left_error) => match self.right.extract(headers).await {
                Ok(extracted) => Ok(Or::Right(extracted)),
                Err(right_error) => Err(OrError {
                    left: left_error,
                    right: right_error,
                }),
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Left: {left}, Right: {right}")]
pub struct OrError<L, R> {
    pub left: L,
    pub right: R,
}

#[cfg(feature = "axum")]
mod axum {
    use axum::response::{IntoResponse, Response};

    use super::OrError;

    impl<L, R> IntoResponse for OrError<L, R>
    where
        L: IntoResponse,
    {
        fn into_response(self) -> Response {
            self.left.into_response()
        }
    }

    impl<L, R> From<OrError<L, R>> for Response
    where
        L: IntoResponse,
    {
        fn from(value: OrError<L, R>) -> Self {
            value.into_response()
        }
    }
}
