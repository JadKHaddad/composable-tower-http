use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use http::Request;
use tower::Service;

use crate::{extract::SealedExtracted, modify::Modifier};

#[derive(Debug, Clone)]
pub struct ModificationService<S, M, T> {
    service: S,
    modifier: M,
    _phantom: PhantomData<T>,
}

impl<S, M, T> ModificationService<S, M, T> {
    pub const fn new(service: S, modifier: M) -> Self {
        Self {
            service,
            modifier,
            _phantom: PhantomData,
        }
    }
}

impl<S, M, B, T> Service<Request<B>> for ModificationService<S, M, T>
where
    M: Modifier<T> + Clone + Send + 'static,
    T: Send + Sync + 'static,
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Future: Send,
    S::Response: From<ModificationError<M::Error>>,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<S::Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        let mut service = self.service.clone();
        let modifier = self.modifier.clone();

        Box::pin(async move {
            match request.extensions_mut().remove::<SealedExtracted<T>>() {
                Some(SealedExtracted(extracted)) => {
                    match modifier.modify(extracted).await {
                        Ok(modified) => request.extensions_mut().insert(SealedExtracted(modified)),
                        Err(err) => return Ok(From::from(ModificationError::Modification(err))),
                    };
                }
                None => return Ok(From::from(ModificationError::Extract)),
            }

            service.call(request).await
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModificationError<E> {
    #[error("Extraction error")]
    Extract,
    #[error("Modification error: {0}")]
    Modification(#[source] E),
}

#[cfg(feature = "axum")]
mod axum {
    use axum::response::{IntoResponse, Response};
    use http::StatusCode;

    use super::ModificationError;

    impl<E> IntoResponse for ModificationError<E>
    where
        E: IntoResponse,
    {
        fn into_response(self) -> Response {
            match self {
                ModificationError::Extract => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
                ModificationError::Modification(err) => err.into_response(),
            }
        }
    }

    impl<E> From<ModificationError<E>> for Response
    where
        E: IntoResponse,
    {
        fn from(value: ModificationError<E>) -> Self {
            value.into_response()
        }
    }
}
