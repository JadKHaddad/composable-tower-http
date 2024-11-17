use super::extractor::Extractor;

#[derive(Debug, Clone)]
pub struct AndExtractor<L, R> {
    left: L,
    right: R,
}

impl<L, R> AndExtractor<L, R> {
    pub const fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}

#[derive(Debug, Clone)]
pub struct And<L, R> {
    pub left: L,
    pub right: R,
}

impl<L, R> Extractor for AndExtractor<L, R>
where
    L: Extractor + Send + Sync,
    R: Extractor + Send + Sync,
{
    type Extracted = And<L::Extracted, R::Extracted>;

    type Error = AndError<L::Error, R::Error>;

    
    async fn extract(&self, headers: &http::HeaderMap) -> Result<Self::Extracted, Self::Error> {
        let left = self.left.extract(headers).await.map_err(AndError::Left)?;
        let right = self.right.extract(headers).await.map_err(AndError::Right)?;

        Ok(And { left, right })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AndError<L, R> {
    #[error("Left: {0}")]
    Left(#[source] L),
    #[error("Right: {0}")]
    Right(#[source] R),
}

#[cfg(feature = "axum")]
mod axum {
    use axum::response::{IntoResponse, Response};

    use super::AndError;

    impl<L, R> IntoResponse for AndError<L, R>
    where
        L: IntoResponse,
        R: IntoResponse,
    {
        fn into_response(self) -> Response {
            match self {
                AndError::Left(err) => err.into_response(),
                AndError::Right(err) => err.into_response(),
            }
        }
    }

    impl<L, R> From<AndError<L, R>> for Response
    where
        L: IntoResponse,
        R: IntoResponse,
    {
        fn from(value: AndError<L, R>) -> Self {
            value.into_response()
        }
    }
}
